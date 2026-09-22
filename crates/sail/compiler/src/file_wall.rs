//! Per-file compile wall. A hang or runaway file fails that file only.
//!
//! The process-global 60s `abort()` is gone: a directory walk continues
//! after a cap so later red files stay visible.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use crate::{CompileError, CompileOpts, CompileOutput};

/// Stable diagnostic id. Registered in the DiagSpec table.
pub const RULE_ID: &str = "sail-file-wall";

/// Cold debug directory builds. Studio's heaviest file is under this.
pub const FILE_WALL_DEBUG: Duration = Duration::from_secs(5);

/// Release / warm incremental. Debug stays at [`FILE_WALL_DEBUG`] so a
/// first compile of a megapage is not clipped by the warm cap.
pub const FILE_WALL_WARM: Duration = Duration::from_secs(2);

/// Product per-file wall: 5s debug, 2s release.
pub fn file_wall() -> Duration {
    if cfg!(debug_assertions) {
        FILE_WALL_DEBUG
    } else {
        FILE_WALL_WARM
    }
}

/// One file exceeded the per-file wall. The worker may still be running;
/// the walk stops waiting on it.
#[derive(Debug, Clone)]
pub struct FileWallTimeout {
    pub path: PathBuf,
    pub wall: Duration,
}

impl FileWallTimeout {
    pub fn into_compile_error(self) -> CompileError {
        CompileError::new(self.message())
            .with_path(self.path)
            .with_category(RULE_ID)
    }

    fn message(&self) -> String {
        format!(
            "compile exceeded {:.0}s per-file wall",
            self.wall.as_secs_f64()
        )
    }
}

impl fmt::Display for FileWallTimeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: [{RULE_ID}] {}", self.path.display(), self.message())
    }
}

impl std::error::Error for FileWallTimeout {}

/// Outcome of one file under [`walk_files`].
#[derive(Debug)]
pub enum FileWorkError<E> {
    Timeout(FileWallTimeout),
    Other(E),
}

impl<E> FileWorkError<E> {
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}

impl From<anyhow::Error> for FileWorkError<anyhow::Error> {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

/// Timeouts collected during a walk, plus the first non-timeout error
/// if the walk stopped on a real compile failure.
#[derive(Debug)]
pub struct WalkFailure<E> {
    pub timeouts: Vec<FileWallTimeout>,
    pub error: Option<E>,
}

/// Run `work` on a worker thread. If it does not finish within `wall`,
/// return [`FileWallTimeout`] naming `path`. The worker is detached.
pub fn run_with_file_wall<T, F>(path: &Path, wall: Duration, work: F) -> Result<T, FileWallTimeout>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("sail-file-wall".into())
        .spawn(move || {
            let _ = tx.send(work());
        })
        .expect("spawn per-file wall worker");
    match rx.recv_timeout(wall) {
        Ok(value) => {
            let _ = handle.join();
            Ok(value)
        }
        Err(RecvTimeoutError::Timeout) => {
            drop(handle);
            Err(FileWallTimeout {
                path: path.to_path_buf(),
                wall,
            })
        }
        Err(RecvTimeoutError::Disconnected) => match handle.join() {
            Ok(()) => Err(FileWallTimeout {
                path: path.to_path_buf(),
                wall,
            }),
            Err(panic) => std::panic::resume_unwind(panic),
        },
    }
}

/// `compile()` under a per-file wall. Timeout becomes [`CompileError`]
/// with [`RULE_ID`] and the source path.
pub fn compile_with_file_wall(
    source: impl Into<String>,
    opts: CompileOpts,
    wall: Duration,
) -> Result<CompileOutput, CompileWithWallError> {
    let source = source.into();
    let path = opts
        .source_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("<input>"));
    match run_with_file_wall(&path, wall, move || crate::compile(&source, opts)) {
        Ok(Ok(out)) => Ok(out),
        Ok(Err(e)) => Err(CompileWithWallError::Compile(e)),
        Err(timeout) => Err(CompileWithWallError::Timeout(timeout)),
    }
}

#[derive(Debug)]
pub enum CompileWithWallError {
    Timeout(FileWallTimeout),
    Compile(CompileError),
}

impl CompileWithWallError {
    pub fn into_compile_error(self) -> CompileError {
        match self {
            Self::Timeout(t) => t.into_compile_error(),
            Self::Compile(e) => e,
        }
    }
}

/// Walk `files`. A per-file timeout is recorded and the walk continues.
/// The first non-timeout error stops the walk (existing fail-fast) but
/// any timeouts already seen travel with it so the cap cannot hide later
/// red files that have already been visited — and files after a timeout
/// still run, which is the file that would have been hidden.
pub fn walk_files<T, E>(
    files: impl IntoIterator<Item = PathBuf>,
    mut work: impl FnMut(&Path) -> Result<T, FileWorkError<E>>,
) -> Result<Vec<T>, WalkFailure<E>> {
    let mut oks = Vec::new();
    let mut timeouts = Vec::new();
    for path in files {
        match work(&path) {
            Ok(v) => oks.push(v),
            Err(FileWorkError::Timeout(t)) => timeouts.push(t),
            Err(FileWorkError::Other(e)) => {
                return Err(WalkFailure {
                    timeouts,
                    error: Some(e),
                });
            }
        }
    }
    if timeouts.is_empty() {
        Ok(oks)
    } else {
        Err(WalkFailure {
            timeouts,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn sleep_double_dies_at_per_file_cap_with_path() {
        let path = PathBuf::from("/tmp/fixtures/hang.sail");
        let err = run_with_file_wall(&path, Duration::from_millis(80), || {
            thread::sleep(Duration::from_secs(10));
        })
        .expect_err("10s sleep must die at the per-file cap");
        let msg = err.to_string();
        assert!(
            msg.contains("hang.sail"),
            "timeout must name the file: {msg}"
        );
        assert!(msg.contains(RULE_ID), "timeout must carry {RULE_ID}: {msg}");
        assert!(
            msg.contains("per-file wall"),
            "timeout must say it is a per-file wall: {msg}"
        );
        let rendered = err.into_compile_error().render("");
        assert!(
            rendered.contains("hang.sail"),
            "CompileError render must name the file: {rendered}"
        );
        assert!(rendered.contains(RULE_ID), "{rendered}");
    }

    #[test]
    fn walk_continues_past_timeout_and_reports_later_red() {
        let visited = Arc::new(AtomicUsize::new(0));
        let files = vec![
            PathBuf::from("aaa-hang.sail"),
            PathBuf::from("bbb-ok.sail"),
            PathBuf::from("ccc-red.sail"),
        ];
        let seen = visited.clone();
        let fail = walk_files(files, |path| {
            seen.fetch_add(1, Ordering::SeqCst);
            let name = path.file_name().unwrap().to_string_lossy();
            if name.starts_with("aaa-hang") {
                Err(FileWorkError::Timeout(FileWallTimeout {
                    path: path.to_path_buf(),
                    wall: Duration::from_millis(50),
                }))
            } else if name.starts_with("ccc-red") {
                Err(FileWorkError::Other("later red: assignments"))
            } else {
                Ok(())
            }
        })
        .expect_err("walk must surface hang + later red");
        assert_eq!(
            visited.load(Ordering::SeqCst),
            3,
            "must visit files after the cap"
        );
        assert_eq!(fail.timeouts.len(), 1);
        assert!(
            fail.timeouts[0].path.ends_with("aaa-hang.sail"),
            "{:?}",
            fail.timeouts[0].path
        );
        assert_eq!(fail.error, Some("later red: assignments"));
    }

    #[test]
    fn walk_does_not_hide_later_file_after_only_timeouts() {
        let visited = Arc::new(AtomicUsize::new(0));
        let files = vec![PathBuf::from("hang.sail"), PathBuf::from("later-ok.sail")];
        let seen = visited.clone();
        let fail = walk_files(files, |path| {
            seen.fetch_add(1, Ordering::SeqCst);
            if path.ends_with("hang.sail") {
                Err(FileWorkError::<()>::Timeout(FileWallTimeout {
                    path: path.to_path_buf(),
                    wall: Duration::from_millis(50),
                }))
            } else {
                Ok(())
            }
        })
        .expect_err("timeouts fail the walk after every file ran");
        assert_eq!(visited.load(Ordering::SeqCst), 2);
        assert!(fail.error.is_none());
        assert_eq!(fail.timeouts.len(), 1);
    }

    #[test]
    fn compile_under_wall_succeeds_for_tiny_source() {
        let src = include_str!("../tests/fixtures/hello.sail");
        let out = compile_with_file_wall(
            src,
            crate::CompileOpts {
                source_path: Some(PathBuf::from("hello.sail")),
                ..crate::CompileOpts::default()
            },
            Duration::from_secs(5),
        )
        .unwrap_or_else(|e| match e {
            CompileWithWallError::Timeout(t) => panic!("tiny fixture hit the wall: {t}"),
            CompileWithWallError::Compile(c) => panic!("{}", c.render(src)),
        });
        assert!(
            out.js.contains("export default"),
            "expected emitted module, got:\n{}",
            out.js
        );
    }

    #[test]
    fn file_wall_debug_is_five_seconds() {
        if cfg!(debug_assertions) {
            assert_eq!(file_wall(), FILE_WALL_DEBUG);
            assert_eq!(FILE_WALL_DEBUG, Duration::from_secs(5));
        } else {
            assert_eq!(file_wall(), FILE_WALL_WARM);
            assert_eq!(FILE_WALL_WARM, Duration::from_secs(2));
        }
    }
}

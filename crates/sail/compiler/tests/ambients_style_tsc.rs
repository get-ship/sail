//! tsc proof for the ambient Style pack.
//!
//! Writes a tiny fixture against the generated `node_modules/.sail/types.d.ts` and asserts:
//! - good Style usage typechecks green
//! - wrong setter / wrong token type is a tsc error
//!
//! Opt-in when `node_modules/.bin/tsc` is present (same stance as synthetic_tsc).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sail_compiler::ambients;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn tsc_path(repo: &Path) -> Option<PathBuf> {
    let p = repo.join("node_modules/.bin/tsc");
    p.exists().then_some(p)
}

#[test]
fn style_misuse_is_tsc_error_and_good_chain_is_green() {
    let repo = repo_root();
    let Some(tsc) = tsc_path(&repo) else {
        eprintln!(
            "SKIP ambients_style_tsc: {} not found — run `bun install` to enable.",
            repo.join("node_modules/.bin/tsc").display()
        );
        return;
    };

    let work = repo.join("target/ambients-style-tsc");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();

    let ambient = ambients::emit_types_dts();
    fs::write(work.join("sail-types.d.ts"), ambient).unwrap();

    // Runtime types for imports inside the ambient pack.
    let runtime = repo
        .join("packages/sail/runtime/src/index.ts")
        .display()
        .to_string()
        .replace('\\', "/");
    let cfg = format!(
        r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "strict": true,
    "noImplicitAny": true,
    "skipLibCheck": true,
    "noEmit": true,
    "paths": {{
      "@ship/sail-runtime": ["{runtime}"],
      "@ship/sail-runtime/*": ["{runtime_parent}/*"]
    }}
  }},
  "include": ["./**/*.ts", "./sail-types.d.ts"]
}}"#,
        runtime = runtime,
        runtime_parent = repo
            .join("packages/sail/runtime/src")
            .display()
            .to_string()
            .replace('\\', "/"),
    );
    fs::write(work.join("tsconfig.json"), cfg).unwrap();

    // Green: known setters + token ambient consts + string form for hyphenated.
    fs::write(
        work.join("good.ts"),
        r#"
const a = pad({ x: 2 }).bg(primary).radius(sm).fg('muted-foreground')
const whenHover = { when: { hover: bg(muted) } }
const matchVariant = { match: { variant: { solid: bg(primary), soft: bg(muted) } } }
const compoundPill = { compound: [{ pill: true }] }
const motionEnter = { animate: { enter: play('fade'), exit: play('fade') } }
const padAndHover = pad(4).bg(primary)
// T0: bound tone + static group.member + string-form paths typecheck as paint args
const tonePaint = bg(tone).fg(tone.foreground).border(tone)
const staticMember = fg(primary.foreground)
const stringPaths = bg('tone' as const).fg('primary.foreground')
const softWash = bg({ color: tone, mix: 15 })
const g = [a, whenHover, matchVariant, compoundPill, motionEnter, padAndHover, tonePaint, staticMember, stringPaths, softWash]
void g
void signal(0)
void controller(() => ({ n: signal(1) }))
"#,
    )
    .unwrap();

    let good = Command::new(&tsc)
        .args(["-p", "tsconfig.json", "--noEmit"])
        .current_dir(&work)
        .output()
        .expect("run tsc good");
    let good_out = format!(
        "{}{}",
        String::from_utf8_lossy(&good.stdout),
        String::from_utf8_lossy(&good.stderr)
    );
    assert!(
        good.status.success(),
        "good Style + ambient usage must typecheck:\n{good_out}"
    );

    // Bad setter
    fs::write(
        work.join("bad-setter.ts"),
        "const x = style.notARealSetter(primary)\nvoid x\n",
    )
    .unwrap();
    // Exclude good; only bad file
    fs::write(
        work.join("tsconfig.bad-setter.json"),
        fs::read_to_string(work.join("tsconfig.json"))
            .unwrap()
            .replace(
                "\"include\": [\"./**/*.ts\", \"./sail-types.d.ts\"]",
                "\"include\": [\"./bad-setter.ts\", \"./sail-types.d.ts\"]",
            ),
    )
    .unwrap();
    let bad_s = Command::new(&tsc)
        .args(["-p", "tsconfig.bad-setter.json", "--noEmit"])
        .current_dir(&work)
        .output()
        .expect("run tsc bad setter");
    let bad_s_out = format!(
        "{}{}",
        String::from_utf8_lossy(&bad_s.stdout),
        String::from_utf8_lossy(&bad_s.stderr)
    );
    assert!(
        !bad_s.status.success(),
        "wrong setter must fail tsc; was green:\n{bad_s_out}"
    );
    assert!(
        bad_s_out.contains("notARealSetter")
            || bad_s_out.contains("2339")
            || bad_s_out.contains("does not exist"),
        "expected property error for bad setter:\n{bad_s_out}"
    );

    // Bad token type (number where color token required)
    fs::write(
        work.join("bad-token.ts"),
        "const x = bg(12345 as 12345)\nvoid x\n",
    )
    .unwrap();
    fs::write(
        work.join("tsconfig.bad-token.json"),
        fs::read_to_string(work.join("tsconfig.json"))
            .unwrap()
            .replace(
                "\"include\": [\"./**/*.ts\", \"./sail-types.d.ts\"]",
                "\"include\": [\"./bad-token.ts\", \"./sail-types.d.ts\"]",
            ),
    )
    .unwrap();
    let bad_t = Command::new(&tsc)
        .args(["-p", "tsconfig.bad-token.json", "--noEmit"])
        .current_dir(&work)
        .output()
        .expect("run tsc bad token");
    let bad_t_out = format!(
        "{}{}",
        String::from_utf8_lossy(&bad_t.stdout),
        String::from_utf8_lossy(&bad_t.stderr)
    );
    assert!(
        !bad_t.status.success(),
        "wrong token type must fail tsc; was green:\n{bad_t_out}"
    );
}

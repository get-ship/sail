//! Pin table for the TypeScript 7 native sidecar tarball.
//!
//! Kept free of crate deps so `build.rs` can `#[path]` this file.

pub const PINS_TOML: &str = include_str!("pins.toml");
pub const TS_NATIVE_VERSION: &str = "7.0.2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub os: String,
    pub arch: String,
    pub npm_package: String,
    pub sha1: String,
    pub ssri: String,
    pub sha256: String,
}

impl Pin {
    pub fn tarball_file_name(&self) -> String {
        format!(
            "{}-{}.tgz",
            self.npm_package.replace('/', "-"),
            TS_NATIVE_VERSION
        )
    }

    pub fn tarball_url(&self) -> String {
        let pkg = &self.npm_package;
        let ver = TS_NATIVE_VERSION;
        let short = pkg.rsplit('/').next().unwrap_or(pkg);
        format!("https://registry.npmjs.org/{pkg}/-/{short}-{ver}.tgz")
    }

    pub fn cargo_home_rel(&self) -> String {
        format!(
            "sail-ts-native/{}/{}",
            TS_NATIVE_VERSION,
            self.tarball_file_name()
        )
    }
}

pub fn parse_pins(src: &str) -> Result<Vec<Pin>, String> {
    let mut pins = Vec::new();
    let mut cur: Option<Pin> = None;
    let mut version: Option<String> = None;
    for (i, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("version") {
            let value = parse_toml_string(rest, i + 1)?;
            version = Some(value);
            continue;
        }
        if line == "[[target]]" {
            if let Some(pin) = cur.take() {
                finish_pin(pin, i + 1, &mut pins)?;
            }
            cur = Some(Pin {
                os: String::new(),
                arch: String::new(),
                npm_package: String::new(),
                sha1: String::new(),
                ssri: String::new(),
                sha256: String::new(),
            });
            continue;
        }
        let Some(eq) = line.find('=') else {
            return Err(format!("pins.toml:{}: expected key = value", i + 1));
        };
        let key = line[..eq].trim();
        let value = parse_toml_string(&line[eq..], i + 1)?;
        let Some(pin) = cur.as_mut() else {
            return Err(format!("pins.toml:{}: key {key} outside [[target]]", i + 1));
        };
        match key {
            "os" => pin.os = value,
            "arch" => pin.arch = value,
            "npm_package" => pin.npm_package = value,
            "sha1" => pin.sha1 = value,
            "ssri" => pin.ssri = value,
            "sha256" => pin.sha256 = value,
            other => return Err(format!("pins.toml:{}: unknown key {other}", i + 1)),
        }
    }
    if let Some(pin) = cur.take() {
        finish_pin(pin, src.lines().count(), &mut pins)?;
    }
    if pins.is_empty() {
        return Err("pins.toml: no [[target]] rows".into());
    }
    if let Some(v) = version {
        if v != TS_NATIVE_VERSION {
            return Err(format!(
                "pins.toml version {v} != TS_NATIVE_VERSION {}",
                TS_NATIVE_VERSION
            ));
        }
    }
    Ok(pins)
}

fn parse_toml_string(eq_and_rest: &str, line: usize) -> Result<String, String> {
    let rest = eq_and_rest.trim().trim_start_matches('=').trim();
    let Some(inner) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) else {
        return Err(format!("pins.toml:{line}: expected quoted string"));
    };
    Ok(inner.to_string())
}

fn finish_pin(pin: Pin, line: usize, out: &mut Vec<Pin>) -> Result<(), String> {
    if pin.os.is_empty()
        || pin.arch.is_empty()
        || pin.npm_package.is_empty()
        || pin.sha256.is_empty()
        || pin.ssri.is_empty()
    {
        return Err(format!("pins.toml:{line}: incomplete [[target]]"));
    }
    if pin.sha256 == "fill" || pin.ssri == "fill" {
        return Err(format!(
            "pins.toml:{line}: sha256/ssri still `fill` for {}-{}",
            pin.os, pin.arch
        ));
    }
    if !pin.ssri.starts_with("sha512-") {
        return Err(format!(
            "pins.toml:{line}: ssri must be sha512-… for {}-{}",
            pin.os, pin.arch
        ));
    }
    out.push(pin);
    Ok(())
}

pub fn pins() -> &'static [Pin] {
    use std::sync::OnceLock;
    static PINS: OnceLock<Vec<Pin>> = OnceLock::new();
    PINS.get_or_init(|| parse_pins(PINS_TOML).expect("pins.toml"))
}

pub fn pin_for(os: &str, arch: &str) -> Option<&'static Pin> {
    pins().iter().find(|p| p.os == os && p.arch == arch)
}

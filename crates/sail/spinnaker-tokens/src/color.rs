//! OKLCH / hex parse, luminance, contrast. One color math for gate + derive.

use anyhow::{Result, bail};

/// WCAG AA normal-text contrast floor.
pub const CONTRAST_FLOOR: f64 = 4.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
    pub alpha: f64,
}

impl Oklch {
    pub fn opaque(l: f64, c: f64, h: f64) -> Self {
        Self {
            l,
            c,
            h,
            alpha: 1.0,
        }
    }

    pub fn to_css(self) -> String {
        if (self.alpha - 1.0).abs() < 1e-6 {
            format!("oklch({:.4} {:.4} {:.2})", self.l, self.c, self.h)
        } else {
            format!(
                "oklch({:.4} {:.4} {:.2} / {:.3})",
                self.l, self.c, self.h, self.alpha
            )
        }
    }

    pub fn mix(self, other: Self, amount: f64) -> Self {
        let t = amount.clamp(0.0, 1.0);
        Self {
            l: self.l + (other.l - self.l) * t,
            c: self.c + (other.c - self.c) * t,
            h: lerp_hue(self.h, other.h, t),
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }

    pub fn with_l(self, l: f64) -> Self {
        Self {
            l: l.clamp(0.0, 1.0),
            ..self
        }
    }
}

fn lerp_hue(a: f64, b: f64, t: f64) -> f64 {
    let mut d = (b - a) % 360.0;
    if d > 180.0 {
        d -= 360.0;
    }
    if d < -180.0 {
        d += 360.0;
    }
    let mut h = (a + d * t) % 360.0;
    if h < 0.0 {
        h += 360.0;
    }
    h
}

/// Parse an authored color: `oklch(...)`, `#rgb`/`#rrggbb`, or a structured
/// `{space,l,c,h}` / `{hex}` object. Returns None for aliases / ops / mix strings.
pub fn parse_color(raw: &str) -> Option<Oklch> {
    let s = raw.trim();
    if s.starts_with('#') {
        return parse_hex(s);
    }
    parse_oklch(s)
}

pub fn parse_oklch(raw: &str) -> Option<Oklch> {
    let s = raw.trim();
    let rest = s.strip_prefix("oklch(")?.strip_suffix(')')?.trim();
    let (body, alpha) = if let Some((lhs, rhs)) = rest.split_once('/') {
        (lhs.trim(), parse_alpha(rhs.trim())?)
    } else {
        (rest, 1.0)
    };
    let parts: Vec<&str> = body.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let l = parse_l(parts[0])?;
    let c: f64 = parts[1].parse().ok()?;
    let h = parse_hue(parts[2..].join(" ").trim())?;
    Some(Oklch { l, c, h, alpha })
}

pub fn parse_hex(s: &str) -> Option<Oklch> {
    let hex = s.strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            (r, g, b, 1.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b, 1.0)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f64 / 255.0;
            (r, g, b, a)
        }
        _ => return None,
    };
    Some(srgb_u8_to_oklch(r, g, b, a))
}

/// Structured color object `{space,l,c,h}` or `{hex}`.
pub fn parse_structured_color(v: &serde_json::Value) -> Option<Oklch> {
    let obj = v.as_object()?;
    if let Some(hex) = obj.get("hex").and_then(|x| x.as_str()) {
        return parse_hex(hex);
    }
    let space = obj.get("space").and_then(|x| x.as_str()).unwrap_or("oklch");
    if space != "oklch" {
        return None;
    }
    let l = num_field(obj.get("l"))?;
    let c = num_field(obj.get("c"))?;
    let h = num_field(obj.get("h"))?;
    let alpha = obj
        .get("a")
        .or_else(|| obj.get("alpha"))
        .map_or(1.0, |x| num_field(Some(x)).unwrap_or(1.0));
    Some(Oklch { l, c, h, alpha })
}

fn num_field(v: Option<&serde_json::Value>) -> Option<f64> {
    match v? {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn parse_l(s: &str) -> Option<f64> {
    if let Some(pct) = s.strip_suffix('%') {
        return Some(pct.parse::<f64>().ok()? / 100.0);
    }
    s.parse().ok()
}

fn parse_alpha(s: &str) -> Option<f64> {
    if let Some(pct) = s.strip_suffix('%') {
        return Some(pct.parse::<f64>().ok()? / 100.0);
    }
    s.parse().ok()
}

fn parse_hue(s: &str) -> Option<f64> {
    if let Ok(h) = s.parse::<f64>() {
        return Some(h);
    }
    let inner = s.strip_prefix("var(")?.trim_end_matches(')').trim();
    let fallback = inner.split(',').nth(1)?.trim();
    fallback.parse().ok()
}

pub fn relative_luminance(c: &Oklch) -> f64 {
    let (r, g, b) = oklch_to_linear_srgb(c.l, c.c, c.h);
    0.2126 * r.max(0.0) + 0.7152 * g.max(0.0) + 0.0722 * b.max(0.0)
}

fn oklch_to_linear_srgb(l: f64, c: f64, h_deg: f64) -> (f64, f64, f64) {
    let h = h_deg.to_radians();
    let a = c * h.cos();
    let b = c * h.sin();
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let bch = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;
    (r, g, bch)
}

fn srgb_u8_to_oklch(r: u8, g: u8, b: u8, alpha: f64) -> Oklch {
    let rl = srgb_u8_to_linear(r);
    let gl = srgb_u8_to_linear(g);
    let bl = srgb_u8_to_linear(b);
    let l_ = (0.4122214708 * rl + 0.5363325363 * gl + 0.0514459929 * bl).cbrt();
    let m_ = (0.2119034982 * rl + 0.6806995451 * gl + 0.1073969566 * bl).cbrt();
    let s_ = (0.0883024619 * rl + 0.2817188376 * gl + 0.6299787005 * bl).cbrt();
    let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let bch = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
    let c = (a * a + bch * bch).sqrt();
    let h = bch.atan2(a).to_degrees().rem_euclid(360.0);
    Oklch { l, c, h, alpha }
}

fn srgb_u8_to_linear(u: u8) -> f64 {
    let x = u as f64 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

pub fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn contrast_pair(fg: &Oklch, bg: &Oklch) -> f64 {
    contrast_ratio(relative_luminance(fg), relative_luminance(bg))
}

/// Pick an opaque text color on `solid` that meets [`CONTRAST_FLOOR`].
pub fn contrast_driven_text(solid: &Oklch) -> Oklch {
    let light = Oklch::opaque(0.99, (solid.c * 0.04).min(0.03), solid.h);
    let dark = Oklch::opaque(0.18, (solid.c * 0.06).min(0.04), solid.h);
    let mut pick = if contrast_pair(&light, solid) >= contrast_pair(&dark, solid) {
        light
    } else {
        dark
    };
    if contrast_pair(&pick, solid) + 1e-6 >= CONTRAST_FLOOR {
        return pick;
    }
    // Walk L toward the far end until the floor is met.
    let toward_white = pick.l > solid.l;
    for step in 0..24 {
        let delta = 0.02 * f64::from(step + 1);
        let l = if toward_white {
            (pick.l + delta).min(0.99)
        } else {
            (pick.l - delta).max(0.05)
        };
        pick = pick.with_l(l);
        if contrast_pair(&pick, solid) + 1e-6 >= CONTRAST_FLOOR {
            break;
        }
    }
    pick
}

/// Floor pair: missing parseable color is a hard error (named).
pub fn require_contrast(
    fg: &Oklch,
    bg: &Oklch,
    fg_name: &str,
    bg_name: &str,
    floor: f64,
    label: &str,
) -> Result<f64> {
    if fg.alpha < 0.99 || bg.alpha < 0.99 {
        bail!(
            "{label}: contrast pair {fg_name} on {bg_name} — translucent color \
             (alpha {fg_a:.2}/{bg_a:.2}); required pairs must be opaque",
            fg_a = fg.alpha,
            bg_a = bg.alpha
        );
    }
    let ratio = contrast_pair(fg, bg);
    if ratio + 1e-6 < floor {
        bail!("{label}: {fg_name} on {bg_name} is {ratio:.2}:1, floor {floor:.1}:1");
    }
    Ok(ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklch_roundtrip_css() {
        let c = parse_oklch("oklch(0.54 0.18 245)").unwrap();
        assert!((c.l - 0.54).abs() < 1e-6);
        let back = parse_oklch(&c.to_css()).unwrap();
        assert!((back.h - 245.0).abs() < 0.1);
    }

    #[test]
    fn hex_parses() {
        let c = parse_hex("#000000").unwrap();
        assert!(c.l < 0.05);
        let w = parse_hex("#ffffff").unwrap();
        assert!(w.l > 0.95);
    }

    #[test]
    fn structured_color() {
        let v = serde_json::json!({ "space": "oklch", "l": 0.5, "c": 0.1, "h": 30 });
        let c = parse_structured_color(&v).unwrap();
        assert!((c.l - 0.5).abs() < 1e-9);
    }

    #[test]
    fn text_on_dark_is_light() {
        let solid = Oklch::opaque(0.40, 0.16, 250.0);
        let text = contrast_driven_text(&solid);
        assert!(text.l > 0.8);
        assert!(contrast_pair(&text, &solid) + 1e-6 >= CONTRAST_FLOOR);
    }

    #[test]
    fn text_on_light_is_dark() {
        let solid = Oklch::opaque(0.86, 0.10, 90.0);
        let text = contrast_driven_text(&solid);
        assert!(text.l < 0.4);
        assert!(contrast_pair(&text, &solid) + 1e-6 >= CONTRAST_FLOOR);
    }

    #[test]
    fn floor_names_pair() {
        let fg = Oklch::opaque(0.85, 0.0, 0.0);
        let bg = Oklch::opaque(0.92, 0.0, 0.0);
        let err = require_contrast(
            &fg,
            &bg,
            "muted-foreground",
            "muted",
            CONTRAST_FLOOR,
            "probe",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("muted-foreground"));
        assert!(err.contains("floor 4.5:1"));
    }
}

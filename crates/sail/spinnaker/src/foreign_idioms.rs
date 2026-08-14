//! Permanent teaching table for another language's habits.
//!
//! CSS properties, Tailwind atoms, and HTML attributes each point at the
//! live form. Unknown names that are not in this table stay generic unknown
//! vocabulary — including our retired author names.

/// Which foreign language the habit comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdiomKind {
    Css,
    Tailwind,
    Html,
}

/// One high-signal habit → live form pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignIdiom {
    /// Author spelling as written (`padding`, `p-4`, `class=`).
    pub habit: &'static str,
    pub kind: IdiomKind,
    /// Live form to write instead (`pad`, `pad(4)`, `style(…)`).
    pub points_at: &'static str,
}

impl ForeignIdiom {
    /// Diagnostic clause naming the live form.
    pub fn teaching_line(&self) -> String {
        let shown = match self.kind {
            IdiomKind::Html if !self.habit.ends_with('=') => {
                // Stored without `=` (`class`); authors write `class=`.
                format!("{}=", self.habit)
            }
            _ => self.habit.to_string(),
        };
        let kind = match self.kind {
            IdiomKind::Css => "CSS",
            IdiomKind::Tailwind => "Tailwind",
            IdiomKind::Html => "HTML",
        };
        format!("{kind} `{shown}` is `{}` here", self.points_at)
    }
}

/// Closed table. High-signal only — not a scraped catalog.
pub static FOREIGN_IDIOMS: &[ForeignIdiom] = &[
    // ── CSS properties ────────────────────────────────────────────────
    idiom("padding", IdiomKind::Css, "pad"),
    idiom("padding-top", IdiomKind::Css, "pad({ t })"),
    idiom("padding-right", IdiomKind::Css, "pad({ r })"),
    idiom("padding-bottom", IdiomKind::Css, "pad({ b })"),
    idiom("padding-left", IdiomKind::Css, "pad({ l })"),
    idiom("padding-inline", IdiomKind::Css, "pad({ x })"),
    idiom("padding-block", IdiomKind::Css, "pad({ y })"),
    idiom("margin", IdiomKind::Css, "margin"),
    idiom("margin-top", IdiomKind::Css, "margin({ t })"),
    idiom("margin-right", IdiomKind::Css, "margin({ r })"),
    idiom("margin-bottom", IdiomKind::Css, "margin({ b })"),
    idiom("margin-left", IdiomKind::Css, "margin({ l })"),
    idiom("background", IdiomKind::Css, "bg"),
    idiom("background-color", IdiomKind::Css, "bg"),
    idiom("color", IdiomKind::Css, "fg"),
    idiom("font-size", IdiomKind::Css, "type({ size })"),
    idiom("font-weight", IdiomKind::Css, "type({ weight })"),
    idiom("font-family", IdiomKind::Css, "type({ family })"),
    idiom("line-height", IdiomKind::Css, "type({ leading })"),
    idiom("letter-spacing", IdiomKind::Css, "type({ tracking })"),
    idiom("text-align", IdiomKind::Css, "type({ align })"),
    idiom("border-width", IdiomKind::Css, "border({ width })"),
    idiom("border-color", IdiomKind::Css, "border({ color })"),
    idiom("border-style", IdiomKind::Css, "border({ style })"),
    idiom("border-radius", IdiomKind::Css, "radius"),
    idiom("width", IdiomKind::Css, "size({ w })"),
    idiom("height", IdiomKind::Css, "size({ h })"),
    idiom("min-width", IdiomKind::Css, "size({ min-w })"),
    idiom("min-height", IdiomKind::Css, "size({ min-h })"),
    idiom("max-width", IdiomKind::Css, "size({ max-w })"),
    idiom("max-height", IdiomKind::Css, "size({ max-h })"),
    idiom("z-index", IdiomKind::Css, "position({ z })"),
    // CSS property string — live form is the plain setter, not a new name.
    idiom("display", IdiomKind::Css, "display"),
    // ── Tailwind atoms (handful, not the catalog) ─────────────────────
    idiom("bg-primary", IdiomKind::Tailwind, "bg(primary)"),
    idiom("p-4", IdiomKind::Tailwind, "pad(4)"),
    idiom("text-sm", IdiomKind::Tailwind, "type({ size: sm })"),
    idiom("flex", IdiomKind::Tailwind, "display(flex)"),
    idiom("items-center", IdiomKind::Tailwind, "<row align>"),
    // ── HTML attributes ───────────────────────────────────────────────
    idiom("class=", IdiomKind::Html, "style(…)"),
    idiom("className=", IdiomKind::Html, "style(…)"),
    idiom("style=", IdiomKind::Html, "style(…)"),
];

const fn idiom(habit: &'static str, kind: IdiomKind, points_at: &'static str) -> ForeignIdiom {
    ForeignIdiom {
        habit,
        kind,
        points_at,
    }
}

/// Look up a written name. `class` / `class=` both hit the HTML row;
/// bare `style` does not — that is our constructor, not the HTML attribute.
pub fn find_foreign_idiom(name: &str) -> Option<&'static ForeignIdiom> {
    let key = normalize_habit(name)?;
    FOREIGN_IDIOMS.iter().find(|row| row.habit == key)
}

/// Teaching clause when `name` is a foreign habit; `None` keeps the generic
/// unknown-vocabulary / unknown-setter wording.
pub fn teaching_line(name: &str) -> Option<String> {
    find_foreign_idiom(name).map(ForeignIdiom::teaching_line)
}

/// Every table row whose habit appears in `src` (utility soup, HTML attrs).
pub fn idioms_in_source(src: &str) -> Vec<&'static ForeignIdiom> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<&'static ForeignIdiom>, row: &'static ForeignIdiom| {
        if !out.iter().any(|existing| existing.habit == row.habit) {
            out.push(row);
        }
    };

    if src.contains("className=") {
        if let Some(row) = find_foreign_idiom("className=") {
            push(&mut out, row);
        }
    } else if src.contains("class=") {
        if let Some(row) = find_foreign_idiom("class=") {
            push(&mut out, row);
        }
    }
    if let Some(rest) = src.find("style=").map(|i| &src[i..]) {
        // `style=` is the HTML attribute. `style({` / `style(` is ours.
        let after = rest.get("style=".len()..).unwrap_or("");
        if !after.starts_with('(') && !after.starts_with('{') {
            if let Some(row) = find_foreign_idiom("style=") {
                push(&mut out, row);
            }
        }
    }

    for word in src.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
        if word.is_empty() {
            continue;
        }
        if let Some(row) = find_foreign_idiom(word) {
            push(&mut out, row);
        }
    }
    out
}

fn normalize_habit(name: &str) -> Option<&'static str> {
    let n = name.trim();
    match n {
        "class" | "class=" => Some("class="),
        "className" | "className=" => Some("className="),
        "style=" => Some("style="),
        // Bare `style` is the product constructor.
        "style" => None,
        other => FOREIGN_IDIOMS
            .iter()
            .find(|row| row.habit == other)
            .map(|row| row.habit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_points_at_pad() {
        let row = find_foreign_idiom("padding").expect("padding is a CSS habit");
        assert_eq!(row.points_at, "pad");
        assert!(
            row.teaching_line().contains("pad"),
            "{}",
            row.teaching_line()
        );
    }

    #[test]
    fn font_size_points_at_type() {
        let row = find_foreign_idiom("font-size").expect("font-size is a CSS habit");
        assert!(
            row.points_at.contains("type"),
            "font-size must point at type: {}",
            row.points_at
        );
    }

    #[test]
    fn class_eq_points_at_style() {
        let row = find_foreign_idiom("class=").expect("class= is an HTML habit");
        assert!(
            row.points_at.contains("style("),
            "class= must point at style(…): {}",
            row.points_at
        );
        assert!(find_foreign_idiom("class").is_some());
    }

    #[test]
    fn tailwind_p4_points_at_pad() {
        let row = find_foreign_idiom("p-4").expect("p-4 is a Tailwind habit");
        assert!(
            row.points_at.contains("pad"),
            "p-4 must point at pad: {}",
            row.points_at
        );
    }

    #[test]
    fn bare_style_is_not_the_html_attr() {
        assert!(find_foreign_idiom("style").is_none());
        assert!(find_foreign_idiom("style=").is_some());
    }

    #[test]
    fn unknown_name_has_no_row() {
        assert!(find_foreign_idiom("notARealHabitZzz").is_none());
        assert!(teaching_line("notARealHabitZzz").is_none());
    }
}

//! Setter → channel assignment (INVENTORY-CHANNELS-1 §3 / job-spec spine).

/// Author channel for a Style setter or motion trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Box,
    Paint,
    Text,
    Animate,
}

impl Channel {
    pub fn key(self) -> &'static str {
        match self {
            Channel::Box => "box",
            Channel::Paint => "paint",
            Channel::Text => "text",
            Channel::Animate => "animate",
        }
    }
}

/// Motion triggers that live under `animate:` (brace-config form).
pub fn is_animate_trigger(name: &str) -> bool {
    matches!(
        name,
        "enter"
            | "exit"
            | "loop"
            | "reveal"
            | "scroll"
            | "hover"
            | "press"
            | "focus"
            | "focus-visible"
    )
}

/// Channel for a product setter name. `None` when unknown / not a paint setter.
///
/// Assignment follows job-spec CHANNELS spine + inventory table. OPEN items
/// home only when the job-spec lists them (objectFit/Position → paint;
/// vertical-align `align` → paint as presentation).
pub fn channel_for_setter(name: &str) -> Option<Channel> {
    Some(match name {
        // box
        "pad" | "padX" | "padY" | "padT" | "padB" | "padL" | "padR" | "padS" | "padE" => {
            Channel::Box
        }
        "m" | "mx" | "my" | "mt" | "mb" | "ms" | "me" | "ml" | "mr" => Channel::Box,
        "minH" | "minW" | "maxH" | "maxW" | "w" | "h" | "size" => Channel::Box,
        "overflow" | "overflowX" | "overflowY" => Channel::Box,
        "display" | "position" | "top" | "bottom" | "left" | "right" | "inset" => Channel::Box,
        "basis" | "self" | "flexDir" | "aspect" | "order" | "justifySelf" | "resize" => {
            Channel::Box
        }
        "gap" | "gapX" | "gapY" => Channel::Box,
        "grow" | "shrink" => Channel::Box,

        // text
        "fontSize" | "fontWeight" | "fontFamily" | "leading" | "tracking" => Channel::Text,
        "whitespace" | "textAlign" | "break" | "tab" | "underline" | "truncate" => Channel::Text,
        "textCase" | "nums" | "italic" | "decoration" | "indent" | "underlineOffset"
        | "antialiased" => Channel::Text,

        // paint (includes job-listed presentation + dual paint forms of scroll/press)
        "bg" | "fg" | "color" | "list" | "shadow" | "opacity" => Channel::Paint,
        "border" | "borderX" | "borderY" | "borderS" | "borderE" | "borderT" | "borderB"
        | "borderR" | "borderL" => Channel::Paint,
        "borderWidth" | "borderBWidth" | "borderTWidth" | "borderEWidth" | "borderSWidth"
        | "borderRWidth" | "borderLWidth" => Channel::Paint,
        "radius" | "radiusT" | "radiusB" | "radiusL" | "radiusR" => Channel::Paint,
        "stroke" | "fill" | "ring" | "outline" | "outlineOffset" => Channel::Paint,
        "translateX" | "translateY" | "z" | "cursor" | "select" | "pointerEvents" => {
            Channel::Paint
        }
        "overscroll" | "appearance" | "content" | "transition" | "duration" | "ease" => {
            Channel::Paint
        }
        "rotate" | "scale" | "tapHighlight" | "saturate" | "group" | "bgImage" | "borderImage"
        | "backdrop" => Channel::Paint,
        "avatarMedia" | "glyphFill" | "scrollbarThumb" | "scrollbarTrack" | "scrollbarThin" => {
            Channel::Paint
        }
        "isolate" | "mixBlend" | "origin" | "textGradient" | "bgGlow" | "srOnly" | "visibility" => {
            Channel::Paint
        }
        "objectFit" | "objectPosition" | "transform" | "caret" | "accent" | "touch" | "scroll" => {
            Channel::Paint
        }
        "willChange" | "contain" | "colorMix" | "colorDeepen" | "controlFrame" | "controlLift"
        | "press" | "boxSizing" | "bgClip" | "animate" | "align" => Channel::Paint,

        _ => return None,
    })
}

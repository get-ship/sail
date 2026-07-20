//! Template IR — the instruction tree the emitter walks.
//!
//! Mirrors `packages/sail/dom/src/template/compile.ts`'s `Instr`
//! type. The emitter's job is to translate each instruction into the
//! matching runtime call.

use crate::diagnostics::Span;
use crate::ir::expr::Expr;

#[derive(Debug, Clone)]
pub enum Instr {
    StaticText { text: String, span: Span },
    DynamicText { expr: Expr },
    Comment { data: String, span: Span },
    Element(ElementInstr),
    If(IfInstr),
    For(ForInstr),
    Match(MatchInstr),
    WhileLoading(WhileLoadingInstr),
    WhileError(WhileErrorInstr),
}

#[derive(Debug, Clone)]
pub struct ElementInstr {
    /// Original tag name (preserved-case, may be a custom element).
    pub tag: String,
    pub attrs: Vec<AttrInstr>,
    pub children: Vec<Instr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileLoadingInstr {
    /// Signal name bound to (e.g. `"agents"`).
    pub bind: String,
    pub body: Vec<Instr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileErrorInstr {
    /// Signal name bound to (e.g. `"agents"`).
    pub bind: String,
    /// Local name for the error value inside the block body.
    pub error_var: String,
    pub body: Vec<Instr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AttrInstr {
    /// `attr="value"` — no expression markers.
    Static { name: String, value: String, span: Span },
    /// `attr={expr}` — single-expression attribute.
    Expr { name: String, expr: Expr, span: Span },
    /// `attr:{nameExpr}={valueExpr}` — dynamic attribute name.
    DynamicAttr { name_expr: Expr, expr: Expr, span: Span },
    /// `bind:value={signal}`.
    BindValue { expr: Expr, span: Span },
    /// `bind:checked={signal}`.
    BindChecked { expr: Expr, span: Span },
    /// `bind:this={varName}` — write a typed handle to the rendered
    /// element into the named scope variable. The expression must be a
    /// bare identifier; refs are write-once-from-runtime, not derivable.
    BindThis { var: String, span: Span },
    /// `<slot reflect-empty="data-attr">` — declarative slot-emptiness
    /// reactivity. Wires a `slotchange` listener that toggles the named
    /// attribute on the host whenever the slot's assigned content
    /// gains or loses visible text. Slot-only; component-only.
    SlotReflectEmpty { attr: String, span: Span },
    /// `on:event|mod[:arg]…={handler}`.
    On { event: String, handler: Expr, modifiers: Vec<Modifier>, span: Span },
    /// `prop:foo={expr}` — assign the value to the element's JS
    /// property `foo` (camelCase). Use when passing complex objects
    /// (signals, arrays, structured data) to a child custom element.
    /// Compiles to `bindProp(el, "foo", () => expr())`.
    Prop { name: String, expr: Expr, span: Span },
    /// `mutation="ship.X.Y"` on a `<form>` — the SDK call chain.
    Mutation { expr: Expr, span: Span },
    /// `motion:trigger={...}` — declarative state-triggered animation.
    /// The config is a static object literal mapping CSS-style property
    /// names to animation preset names. Compiles to a pre-baked
    /// `element.animate(...)` call wired to the trigger's DOM events.
    Motion {
        trigger: String,
        props: Vec<MotionProp>,
        /// `stagger: '40ms'` — the per-sibling delay STEP, in milliseconds.
        /// Orchestration is what separates a list that appears from a list that
        /// arrives. It is a reserved key in the motion config, not a CSS property,
        /// because it describes the relationship BETWEEN elements rather than
        /// anything about one of them.
        stagger_ms: Option<u32>,
        /// Config-level timing that applies to every channel unless the
        /// channel overrides it. This is the layer the duration tokens used
        /// to occupy: `duration: 3600` says in one number what
        /// `--duration-cinematic` needed a global token to say.
        timing: MotionTiming,
        span: Span,
    },
    /// `use:action` or `use:action={param}` — attach a reusable behavior
    /// (an "action") to the rendered element. `action` resolves to a value
    /// in scope (a hook handle that is itself use-able, or an imported
    /// action fn); `param` is the optional reactive parameter. Compiles to
    /// a runtime `runAction(el, action, () => param)` call that invokes the
    /// action with the element on mount, calls its `update(param)` when the
    /// param dependencies change, and `destroy()` on unmount. This is the
    /// first-class element→behavior primitive: it removes author-facing
    /// `ref` + `effect` for the whole element-behavior category.
    Use {
        action: String,
        param: Option<Expr>,
        span: Span,
    },
 }

/// One animated property within a `motion:` config. The optional preset
/// name resolves against `crates/sail/lint/animation-presets.json` at emit
/// time. Multiple `MotionProp`s in the same `motion:` directive share a
/// trigger and fire from the same DOM event pair, but each becomes its own
/// `animate(...)` call.
///
/// A preset is a NAMED DEFAULT, not a cage. `scale: 'pop'` and
/// `scale: { from: 1, to: 1.75 }` are both legal, and
/// `scale: { preset: 'pop', to: 1.9 }` starts from the preset and overrides
/// one field. `preset` is `None` exactly when the author wrote a bare
/// inline object — the magnitudes then come from the object (falling back
/// to the runtime's rest/final values per property).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionProp {
    pub property: String,
    pub preset: Option<String>,
    /// Inline magnitudes. `None` defers to the preset, then to the
    /// runtime's per-property rest/final value.
    pub from: Option<MotionValue>,
    pub to: Option<MotionValue>,
    /// Per-channel timing overrides — these beat the config-level
    /// `timing`, which in turn beats the preset's baked timing.
    pub timing: MotionTiming,
}

/// Timing fields shared by the config level and the per-channel level.
/// Every field is optional; resolution is preset < config < channel.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MotionTiming {
    /// Milliseconds. A bare number IS milliseconds — the unit is not a
    /// question anyone should have to ask on an animation surface.
    pub duration: Option<MotionValue>,
    pub delay: Option<MotionValue>,
    /// A named easing keyword or an inline `[x1, y1, x2, y2]` bezier.
    pub easing: Option<MotionValue>,
    pub iterations: Option<MotionValue>,
    pub direction: Option<MotionValue>,
}

impl MotionTiming {
    pub fn is_empty(&self) -> bool {
        self.duration.is_none()
            && self.delay.is_none()
            && self.easing.is_none()
            && self.iterations.is_none()
            && self.direction.is_none()
    }

    /// Every expression this timing block carries, for the passes that
    /// need to walk expressions (scope-binding, dynamic detection).
    pub fn exprs(&self) -> Vec<&Expr> {
        [
            self.duration.as_ref(),
            self.delay.as_ref(),
            self.easing.as_ref(),
            self.iterations.as_ref(),
            self.direction.as_ref(),
        ]
        .into_iter()
        .flatten()
        .filter_map(MotionValue::as_expr)
        .collect()
    }
}

impl MotionProp {
    /// Every expression on this channel, config-field order.
    pub fn exprs(&self) -> Vec<&Expr> {
        let mut out: Vec<&Expr> = [self.from.as_ref(), self.to.as_ref()]
            .into_iter()
            .flatten()
            .filter_map(MotionValue::as_expr)
            .collect();
        out.extend(self.timing.exprs());
        out
    }
}

/// A single value inside a `motion:` config.
///
/// The animation surface is where numbers are NATIVE — we bundled a real
/// animation library to build real animation, and a duration is a number of
/// milliseconds, not a global design token. `sx` stays closed because
/// colour and spacing are theming integrity; a 3600ms loop is not.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionValue {
    /// A bare number: milliseconds for time fields, a channel-native
    /// magnitude otherwise (px for translate, deg for rotate, unitless
    /// for scale/opacity).
    Num(f64),
    /// A quoted string — a preset name, an easing keyword, or a
    /// unit-bearing magnitude like `'120px'` / `'-12%'` / `'15deg'`.
    Str(String),
    /// An inline cubic-bezier `[0.6, 0, 0.2, 1]`.
    Bezier([f64; 4]),
    /// A reactive expression — `duration: {speed()}`. Emitted as a thunk
    /// and re-read inside an effect, so the animation follows the signal.
    Expr(Expr),
}

impl MotionValue {
    pub fn as_expr(&self) -> Option<&Expr> {
        match self {
            MotionValue::Expr(e) => Some(e),
            _ => None,
        }
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, MotionValue::Expr(_))
    }
}

#[derive(Debug, Clone)]
pub enum Modifier {
    PreventDefault,
    StopPropagation,
    StopImmediatePropagation,
    SelfMod,
    Trusted,
    Key(Option<String>),
    Debounce(Option<String>),
    Throttle(Option<String>),
}

#[derive(Debug, Clone)]
pub struct IfInstr {
    pub branches: Vec<IfBranch>,
    pub else_body: Option<Vec<Instr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub cond: Expr,
    pub body: Vec<Instr>,
}

#[derive(Debug, Clone)]
pub struct ForInstr {
    pub var_name: String,
    pub index_name: Option<String>,
    pub iterable: Expr,
    pub key: Option<Expr>,
    pub body: Vec<Instr>,
    pub else_body: Option<Vec<Instr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchInstr {
    pub discriminant: Expr,
    pub cases: Vec<MatchCase>,
    pub default: Option<Vec<Instr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub test: Expr,
    pub body: Vec<Instr>,
}

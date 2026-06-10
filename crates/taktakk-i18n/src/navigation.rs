//! Direction-aware navigation: which arrow direction means "forward"?
//!
//! In RTL locales the Back/Next arrows are mirrored, but pictograms for
//! universal concepts (emergency exit, water, heart) must NOT be mirrored.
//!
//! RFC 015 §2.2: "Must Mirror" vs "Must Not Mirror" rules.

use crate::direction::TextDirection;

/// Navigation arrow orientation for the current text direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationArrows {
    /// CSS logical direction for the "forward/next" arrow icon.
    pub forward: ArrowDir,
    /// CSS logical direction for the "back/previous" arrow icon.
    pub back: ArrowDir,
}

/// Physical arrow direction (for SVG icon rotation or CSS transform).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDir {
    Right,
    Left,
}

impl NavigationArrows {
    /// Derive arrow directions from the active text direction.
    pub fn for_direction(dir: TextDirection) -> Self {
        match dir {
            TextDirection::Ltr => Self { forward: ArrowDir::Right, back: ArrowDir::Left },
            TextDirection::Rtl => Self { forward: ArrowDir::Left,  back: ArrowDir::Right },
        }
    }
}

/// CSS `dir` attribute value for use on the root element.
pub fn root_dir_attr(dir: TextDirection) -> &'static str {
    dir.html_attr()
}

/// Whether a given icon category should be mirrored in RTL.
///
/// According to RFC 015 §2.2, directional UI icons mirror; universal
/// safety/pictogram icons do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconMirrorPolicy {
    /// Mirror this icon in RTL (e.g. arrows, back/forward chevrons).
    Mirror,
    /// Never mirror this icon (e.g. emergency exit, water drop, heart).
    NeverMirror,
}

/// Determine the mirror policy for common icon classes.
pub fn icon_mirror_policy(icon_class: &str) -> IconMirrorPolicy {
    match icon_class {
        "arrow-back" | "arrow-forward" | "chevron-left" | "chevron-right"
        | "progress-fill" | "list-indent" => IconMirrorPolicy::Mirror,
        _ => IconMirrorPolicy::NeverMirror,
    }
}

use nsh_base::theme;
use nsh_core::prelude::*;

#[must_use]
pub fn success() -> AdaptiveColor {
    theme().color("success").unwrap_or_else(|| theme().primary)
}

#[must_use]
pub fn warning() -> AdaptiveColor {
    theme()
        .color("warning")
        .unwrap_or_else(|| theme().destructive)
}

#[must_use]
pub fn radius_control() -> Points {
    theme().radius
}

#[must_use]
pub fn radius_card() -> Points {
    px(theme().number("radius-card").unwrap_or(12.0))
}

#[must_use]
pub fn radius_inset() -> Points {
    px(theme().number("radius-inset").unwrap_or(6.0))
}

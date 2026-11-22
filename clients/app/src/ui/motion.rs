use nsh_core::prelude::*;

pub const INSTANT_MS: f32 = 120.0;
pub const FAST_MS: f32 = 180.0;
pub const BASE_MS: f32 = 240.0;

pub const PRESS_SCALE: f32 = 0.97;

pub fn apply_hover_transition<E: Styled>(s: E) -> E {
    if reduce_motion() {
        return s.transition_none();
    }
    s.transition_opacity().duration(INSTANT_MS).ease_out()
}

pub fn apply_press_transition<E: Styled>(s: E) -> E {
    if reduce_motion() {
        return s.transition_none();
    }
    s.transition_transform()
        .duration(FAST_MS)
        .ease_bezier(0.22, 1.0, 0.36, 1.0)
}

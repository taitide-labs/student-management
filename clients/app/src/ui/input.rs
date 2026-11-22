use nsh_base::{input, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, platform, token, typography};

pub fn app_input(placeholder: &'static str, value: Signal<String>, masked: bool) -> Input {
    build_input(
        placeholder,
        move || value.get(),
        move |next: &str| value.set(next.to_owned()),
        masked,
    )
}

pub fn controlled_input(
    placeholder: &'static str,
    value: impl Fn() -> String + 'static,
    on_change: impl Fn(&str) + 'static,
    masked: bool,
) -> Input {
    build_input(placeholder, value, on_change, masked)
}

fn build_input(
    placeholder: &'static str,
    value: impl Fn() -> String + 'static,
    on_change: impl Fn(&str) + 'static,
    masked: bool,
) -> Input {
    let terminal = platform::is_terminal();
    let height = if platform::is_mobile() {
        px(44.0)
    } else {
        px(32.0)
    };
    input()
        .value(value)
        .on_change(on_change)
        .placeholder(placeholder)
        .placeholder_color(|| color(theme().muted_foreground))
        .caret_color(|| color(theme().primary))
        .masked(masked)
        .h(height)
        .px_3()
        .text_sm()
        .text_color(move || theme().foreground)
        .bg(move || theme().background)
        .rounded(move || {
            if terminal {
                px(0.0)
            } else {
                token::radius_control()
            }
        })
        .border_1()
        .border_color(move || theme().input)
        .map(motion::apply_hover_transition)
        .focus(|s| s.border_color(move || theme().ring))
}

pub fn labeled_field(
    label: &'static str,
    control: impl IntoElement,
    error: impl Fn() -> Option<SharedString> + 'static,
) -> impl IntoElement {
    let error_memo = computed(error);
    view()
        .gap_1p5()
        .child(typography::muted_text(label))
        .child(control)
        .child(when(
            move || error_memo.with(Option::is_some),
            move || typography::error_text(move || error_memo.get().unwrap_or_default()),
        ))
}

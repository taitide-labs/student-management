use nsh_base::{button, theme};
use nsh_core::{element::IntoProp, prelude::*};

use crate::ui::{motion, platform, token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Default,
    Outline,
    Ghost,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSize {
    Sm,
    Md,
    Lg,
}

impl ControlSize {
    fn height(self) -> Points {
        if platform::is_mobile() {
            return match self {
                Self::Sm => px(40.0),
                Self::Md => px(44.0),
                Self::Lg => px(48.0),
            };
        }
        match self {
            Self::Sm => px(28.0),
            Self::Md => px(32.0),
            Self::Lg => px(38.0),
        }
    }

    fn px_space(self) -> Points {
        match self {
            Self::Sm => px(10.0),
            Self::Md => px(14.0),
            Self::Lg => px(18.0),
        }
    }
}

pub fn app_button<M>(
    label: impl IntoReactive<SharedString>,
    variant: ButtonVariant,
    size: ControlSize,
    disabled: impl IntoProp<bool, M>,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    build_button(label, None, variant, size, disabled, on_press)
}

pub fn app_button_accessible<M>(
    label: impl IntoReactive<SharedString>,
    accessibility_label: impl Into<SharedString>,
    variant: ButtonVariant,
    size: ControlSize,
    disabled: impl IntoProp<bool, M>,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    build_button(
        label,
        Some(accessibility_label.into()),
        variant,
        size,
        disabled,
        on_press,
    )
}

fn build_button<M>(
    label: impl IntoReactive<SharedString>,
    accessibility_label: Option<SharedString>,
    variant: ButtonVariant,
    size: ControlSize,
    disabled: impl IntoProp<bool, M>,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    let terminal = platform::is_terminal();
    let mut btn = button()
        .disabled(disabled)
        .on_press(on_press)
        .flex_shrink_0()
        .h(size.height())
        .px(size.px_space())
        .items_center()
        .justify_center()
        .rounded(move || {
            if terminal {
                px(0.0)
            } else {
                token::radius_control()
            }
        })
        .map(motion::apply_press_transition)
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.45))
        .when(!reduce_motion() && !terminal, |b| {
            b.active(|s| s.scale(motion::PRESS_SCALE).opacity(0.9))
        })
        .when(reduce_motion() || terminal, |b| {
            b.active(|s| s.opacity(0.75))
        });
    if let Some(a11y) = accessibility_label {
        btn = btn.accessibility_label(a11y);
    }

    apply_variant(btn, variant, terminal).child(
        text(label)
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .number_of_lines(1)
            .text_color(move || variant_fg(variant)),
    )
}

pub fn sidebar_nav_button(
    label: &'static str,
    selected: impl Fn() -> bool + Copy + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .selected(selected)
        .accessibility_label(label)
        .on_press(on_press)
        .h(px(32.0))
        .px_3()
        .flex_row()
        .items_center()
        .rounded(token::radius_inset)
        .map(motion::apply_hover_transition)
        .hover(|s| s.bg(move || theme().sidebar_accent))
        .selected_style(|s| s.bg(move || theme().sidebar_accent))
        .focus_visible(|s| {
            s.outline(px(2.0))
                .outline_color(move || theme().sidebar_ring)
        })
        .child(
            text(label)
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || {
                    if selected() {
                        theme().sidebar_accent_foreground
                    } else {
                        theme().sidebar_foreground
                    }
                }),
        )
}

pub fn compact_nav_pill(
    label: &'static str,
    selected: impl Fn() -> bool + Copy + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .selected(selected)
        .accessibility_label(label)
        .on_press(on_press)
        .flex_shrink_0()
        .h(px(28.0))
        .px_2p5()
        .items_center()
        .justify_center()
        .rounded(token::radius_inset)
        .selected_style(|s| s.bg(move || theme().accent))
        .child(
            text(label)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || {
                    if selected() {
                        theme().primary
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}

pub fn mobile_tab_button(
    label: &'static str,
    selected: impl Fn() -> bool + Copy + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .selected(selected)
        .accessibility_label(label)
        .on_press(on_press)
        .flex_1()
        .min_h(px(48.0))
        .items_center()
        .justify_center()
        .selected_style(|s| s.bg(move || theme().accent))
        .child(
            text(label)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || {
                    if selected() {
                        theme().primary
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}

pub fn terminal_nav_row(
    index: usize,
    label: &'static str,
    selected: impl Fn() -> bool + Copy + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .selected(selected)
        .accessibility_label(label)
        .on_press(on_press)
        .px_2()
        .py_1()
        .selected_style(|s| s.bg(move || theme().accent))
        .child(
            text(format!("[{}] {}", index + 1, label))
                .text_sm()
                .text_color(move || {
                    if selected() {
                        theme().primary
                    } else {
                        theme().foreground
                    }
                }),
        )
}

fn apply_variant(
    el: nsh_base::ButtonElement,
    variant: ButtonVariant,
    terminal: bool,
) -> nsh_base::ButtonElement {
    match variant {
        ButtonVariant::Primary => el.bg(move || theme().primary).hover(|s| s.opacity(0.92)),
        ButtonVariant::Default => el
            .bg(move || theme().secondary)
            .hover(|s| s.bg(move || theme().accent)),
        ButtonVariant::Outline => el
            .border_1()
            .border_color(move || theme().border)
            .bg(move || theme().background)
            .hover(|s| s.bg(move || theme().muted)),
        ButtonVariant::Ghost => el
            .when(terminal, |v| {
                v.border_1().border_color(move || theme().border)
            })
            .hover(|s| s.bg(move || theme().muted)),
        ButtonVariant::Destructive => el
            .border_1()
            .border_color(move || theme().destructive)
            .bg(move || theme().background)
            .hover(|s| s.bg(move || theme().destructive.opacity(0.12))),
    }
}

fn variant_fg(variant: ButtonVariant) -> AdaptiveColor {
    match variant {
        ButtonVariant::Primary => theme().primary_foreground,
        ButtonVariant::Default => theme().secondary_foreground,
        ButtonVariant::Outline | ButtonVariant::Ghost => theme().foreground,
        ButtonVariant::Destructive => theme().destructive,
    }
}

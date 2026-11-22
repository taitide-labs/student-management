use nsh_base::theme;
use nsh_core::prelude::*;

pub fn page_title(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_base()
        .font_weight(FontWeight::SEMIBOLD)
        .role(AccessibilityRole::Header)
        .text_color(move || theme().foreground)
}

pub fn section_title(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .role(AccessibilityRole::Header)
        .text_color(move || theme().foreground)
}

pub fn body_text(label: impl IntoReactive<SharedString>) -> Text {
    text(label).text_sm().text_color(move || theme().foreground)
}

pub fn body_medium(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(move || theme().foreground)
}

pub fn muted_text(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_xs()
        .text_color(move || theme().muted_foreground)
}

pub fn mono_text(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .tabular_nums()
        .text_color(move || theme().foreground)
}

pub fn mono_muted(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_xs()
        .tabular_nums()
        .text_color(move || theme().muted_foreground)
}

pub fn error_text(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_xs()
        .text_color(move || theme().destructive)
}

pub fn sidebar_user_name(label: impl IntoReactive<SharedString>) -> Text {
    text(label)
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(move || theme().sidebar_foreground)
}

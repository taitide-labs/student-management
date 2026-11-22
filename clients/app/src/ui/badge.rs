use nsh_base::theme;
use nsh_core::prelude::*;

use crate::ui::{platform, token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeTone {
    Neutral,
    Primary,
    Success,
    Warning,
    Destructive,
}

pub fn badge(label: impl IntoReactive<SharedString>, tone: BadgeTone) -> impl IntoElement {
    let terminal = platform::is_terminal();
    view()
        .px_2()
        .py_0p5()
        .self_start()
        .flex_shrink_0()
        .rounded(move || {
            if terminal {
                px(0.0)
            } else {
                token::radius_inset()
            }
        })
        .border_1()
        .border_color(move || tone_border(tone))
        .bg(move || tone_bg(tone))
        .child(
            text(label)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || tone_fg(tone)),
        )
}

fn tone_bg(tone: BadgeTone) -> AdaptiveColor {
    match tone {
        BadgeTone::Neutral => theme().muted,
        BadgeTone::Primary => theme().accent,
        BadgeTone::Success => token::success().opacity(0.14),
        BadgeTone::Warning => token::warning().opacity(0.14),
        BadgeTone::Destructive => theme().destructive.opacity(0.14),
    }
}

fn tone_border(tone: BadgeTone) -> AdaptiveColor {
    match tone {
        BadgeTone::Neutral => theme().border,
        BadgeTone::Primary => theme().primary.opacity(0.35),
        BadgeTone::Success => token::success().opacity(0.35),
        BadgeTone::Warning => token::warning().opacity(0.35),
        BadgeTone::Destructive => theme().destructive.opacity(0.35),
    }
}

fn tone_fg(tone: BadgeTone) -> AdaptiveColor {
    match tone {
        BadgeTone::Neutral => theme().muted_foreground,
        BadgeTone::Primary => theme().primary,
        BadgeTone::Success => token::success(),
        BadgeTone::Warning => token::warning(),
        BadgeTone::Destructive => theme().destructive,
    }
}

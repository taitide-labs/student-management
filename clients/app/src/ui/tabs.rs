use nsh_base::{tab, tabs, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, platform, token};

pub fn segmented_tabs<T: Copy + PartialEq + 'static>(
    options: Vec<(T, &'static str)>,
    current: impl Fn() -> T + Copy + 'static,
    on_select: impl Fn(T) + Clone + 'static,
) -> impl IntoElement {
    let terminal = platform::is_terminal();
    let tab_nodes = options.into_iter().map(move |(value, label)| {
        let is_selected = move || current() == value;
        let select_cb = on_select.clone();
        tab()
            .selected(is_selected)
            .on_press(move |_: &()| select_cb(value))
            .flex_shrink_0()
            .px_2p5()
            .py_1p5()
            .rounded(move || {
                if terminal {
                    px(0.0)
                } else {
                    token::radius_inset()
                }
            })
            .child(
                text(label)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(move || {
                        if is_selected() {
                            theme().foreground
                        } else {
                            theme().muted_foreground
                        }
                    }),
            )
    });

    tabs()
        .flex_row()
        .self_start()
        .gap_1()
        .p_1()
        .rounded(move || {
            if terminal {
                px(0.0)
            } else {
                token::radius_control()
            }
        })
        .bg(move || theme().muted)
        .indicator_style(move |s| {
            let base = s
                .top_1()
                .bottom_1()
                .rounded(move || {
                    if terminal {
                        px(0.0)
                    } else {
                        token::radius_inset()
                    }
                })
                .bg(move || theme().card)
                .border_1()
                .border_color(move || theme().border);
            if reduce_motion() {
                base.transition_none()
            } else {
                base.transition_transform()
                    .duration(motion::BASE_MS)
                    .ease_in_out()
            }
        })
        .children(tab_nodes)
}

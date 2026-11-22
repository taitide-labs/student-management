use nsh_base::{button, theme};
use nsh_core::prelude::*;

use crate::ui::{motion, platform, typography};

pub fn table_header_row(compact: bool, content: impl IntoElement) -> impl IntoElement {
    view()
        .h(px(32.0))
        .px(if compact { px(16.0) } else { px(24.0) })
        .flex_row()
        .items_center()
        .gap(if compact { px(8.0) } else { px(12.0) })
        .border_b_1()
        .border_color(move || theme().border)
        .bg(move || theme().background)
        .child(content)
}

pub fn table_row_button(
    compact: bool,
    on_press: impl Fn() + 'static,
    content: impl IntoElement,
) -> impl IntoElement {
    let min_h = if platform::is_mobile() {
        px(52.0)
    } else {
        px(40.0)
    };
    button()
        .w_full()
        .on_press(on_press)
        .min_h(min_h)
        .px(if compact { px(16.0) } else { px(24.0) })
        .py_2()
        .flex_row()
        .items_center()
        .gap(if compact { px(8.0) } else { px(12.0) })
        .border_b_1()
        .border_color(move || theme().border)
        .map(motion::apply_hover_transition)
        .hover(|s| s.bg(move || theme().muted))
        .active(|s| s.bg(move || theme().accent))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(content)
}

pub fn table_col_head(label: &'static str) -> impl IntoElement {
    typography::muted_text(label)
}

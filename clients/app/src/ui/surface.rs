use nsh_base::theme;
use nsh_core::{element::desktop::title_bar, prelude::*};

use crate::ui::{platform, token, typography};

pub fn app_canvas() -> View {
    view()
        .flex_1()
        .min_h(px(0.0))
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
}

pub fn desktop_window_bar(
    has_sidebar: bool,
    leading: impl IntoElement,
    trailing: impl IntoElement,
) -> impl IntoElement {
    view()
        .h(move || {
            let top = title_bar().top;
            if top > px(52.0) { top } else { px(52.0) }
        })
        .pl(move || {
            if has_sidebar {
                px(24.0)
            } else {
                title_bar().left + px(16.0)
            }
        })
        .pr(move || if has_sidebar { px(24.0) } else { px(16.0) })
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .bg(move || theme().background)
        .border_b_1()
        .border_color(move || theme().border)
        .window_drag(true)
        .child(leading)
        .child(view().window_drag(false).flex_shrink_0().child(trailing))
}

pub fn sidebar_frame(
    brand: &'static str,
    nav_items: impl IntoElement,
    footer: impl IntoElement,
) -> impl IntoElement {
    view()
        .w(px(220.0))
        .min_h(px(0.0))
        .bg(move || theme().sidebar)
        .border_r_1()
        .border_color(move || theme().sidebar_border)
        .child(
            view()
                .h(move || {
                    let top = title_bar().top;
                    if top > px(52.0) { top } else { px(52.0) }
                })
                .pl(move || title_bar().left + px(16.0))
                .pr_4()
                .flex_row()
                .items_center()
                .window_drag(true)
                .child(
                    text(brand)
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().sidebar_foreground),
                ),
        )
        .child(view().flex_1().px_2().py_2().gap_1().child(nav_items))
        .child(divider())
        .child(view().p_3().gap_2().child(footer))
}

pub fn compact_nav_strip(nav_items: impl IntoElement) -> impl IntoElement {
    view()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap_1()
        .px_4()
        .py_2()
        .bg(move || theme().sidebar)
        .border_b_1()
        .border_color(move || theme().sidebar_border)
        .child(nav_items)
}

pub fn mobile_tab_strip(tab_items: impl IntoElement) -> impl IntoElement {
    view()
        .flex_row()
        .pb_safe_area()
        .bg(move || theme().card)
        .border_t_1()
        .border_color(move || theme().border)
        .child(tab_items)
}

pub fn terminal_frame(
    top_bar: impl IntoElement,
    left_nav: impl IntoElement,
    body: impl IntoElement,
    hint_bar: impl IntoElement,
) -> impl IntoElement {
    view()
        .flex_1()
        .min_h(px(0.0))
        .child(
            view()
                .flex_row()
                .justify_between()
                .px_2()
                .py_1()
                .border_b_1()
                .border_color(move || theme().border)
                .child(top_bar),
        )
        .child(
            view()
                .flex_1()
                .flex_row()
                .min_h(px(0.0))
                .child(
                    view()
                        .w(px(160.0))
                        .border_r_1()
                        .border_color(move || theme().border)
                        .p_2()
                        .gap_1()
                        .child(left_nav),
                )
                .child(view().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(body)),
        )
        .child(
            view()
                .px_2()
                .py_1()
                .border_t_1()
                .border_color(move || theme().border)
                .child(hint_bar),
        )
}

pub fn filter_strip(content: impl IntoElement) -> impl IntoElement {
    view()
        .px(move || {
            if window().width < px(720.0) {
                px(16.0)
            } else {
                px(24.0)
            }
        })
        .py_2p5()
        .flex_row()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_3()
        .bg(move || theme().background)
        .border_b_1()
        .border_color(move || theme().border)
        .child(content)
}

pub fn card() -> View {
    let terminal = platform::is_terminal();
    view()
        .p_4()
        .gap_3()
        .rounded(move || {
            if terminal {
                px(0.0)
            } else {
                token::radius_card()
            }
        })
        .border_1()
        .border_color(move || theme().border)
        .bg(move || theme().card)
}

pub fn divider() -> View {
    view().h(px(1.0)).w_full().bg(move || theme().border)
}

pub fn section_header(
    title: impl IntoReactive<SharedString>,
    subtitle: impl IntoReactive<SharedString>,
) -> impl IntoElement {
    view()
        .gap_1()
        .child(typography::section_title(title))
        .child(typography::muted_text(subtitle))
}

pub fn empty_state(
    title: impl IntoReactive<SharedString>,
    hint: impl IntoReactive<SharedString>,
) -> impl IntoElement {
    view()
        .items_center()
        .justify_center()
        .py_10()
        .px_6()
        .gap_1p5()
        .child(typography::body_medium(title))
        .child(typography::muted_text(hint))
}

pub fn skeleton_bar(width: Points, height: Points) -> View {
    view()
        .w(width)
        .h(height)
        .rounded(token::radius_inset)
        .bg(move || theme().muted)
}

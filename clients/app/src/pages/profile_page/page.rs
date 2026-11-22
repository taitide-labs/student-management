use nsh_core::prelude::*;
use student_management_gateway::contract::AccountSession;

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button},
    surface::{card, desktop_window_bar, divider},
    typography,
};

pub fn profile_page(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    is_dark: impl Fn() -> bool + Copy + 'static,
    on_toggle_theme: impl Fn() + Copy + 'static,
    on_sign_out: impl Fn() + Clone + 'static,
) -> impl IntoElement {
    let is_wide = computed(|| window().width >= px(720.0));
    view()
        .flex_1()
        .min_h(px(0.0))
        .child(when(
            move || is_wide.get(),
            || desktop_window_bar(true, typography::page_title("个人"), view()),
        ))
        .child(
            scroll()
                .flex_1()
                .child(view().p_6().gap_4().child(card().child(dynamic(move || {
                    let sign_out = on_sign_out.clone();
                    match session() {
                        None => typography::muted_text("未登录").into_element(),
                        Some(active) => view()
                            .gap_3()
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(typography::section_title(active.display_name))
                                    .child(typography::mono_muted(format!("@{}", active.username))),
                            )
                            .child(typography::muted_text(format!(
                                "当前角色 · {}",
                                active.role.label()
                            )))
                            .child(divider())
                            .child(
                                view()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap_2()
                                    .child(app_button(
                                        move || {
                                            if is_dark() {
                                                "切换浅色"
                                            } else {
                                                "切换深色"
                                            }
                                        },
                                        ButtonVariant::Outline,
                                        ControlSize::Md,
                                        false,
                                        on_toggle_theme,
                                    ))
                                    .child(app_button(
                                        "退出登录",
                                        ButtonVariant::Destructive,
                                        ControlSize::Md,
                                        false,
                                        sign_out,
                                    )),
                            )
                            .into_element(),
                    }
                })))),
        )
}

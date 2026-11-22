use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, Role};

use crate::{
    app::{
        init,
        nav::{self, NavDestination},
    },
    features::{
        enrollment,
        identity::{parts::role_badge, store as identity_store},
    },
    pages::{
        accounts_page::page::accounts_page, grading_page::page::grading_page,
        homerooms_page::page::homerooms_page, login_page::page::login_page,
        profile_page::page::profile_page, student_detail_page::page::student_detail_page,
        students_page::page::students_page,
    },
    ui::{
        button::{
            ButtonVariant, ControlSize, app_button, compact_nav_pill, mobile_tab_button,
            sidebar_nav_button, terminal_nav_row,
        },
        navigation::stacked_shell,
        platform,
        surface::{
            app_canvas, compact_nav_strip, desktop_window_bar, mobile_tab_strip, sidebar_frame,
            terminal_frame,
        },
        toast::toast_host,
        typography,
    },
};

/// 学生管理系统主应用根组件。
#[component]
pub fn App() -> impl IntoElement {
    init::configure_gateway();
    init::configure_window();
    bind_shortcuts();

    app_canvas()
        .child(dynamic(|| match identity_store::current_session() {
            None => untracked(|| login_page().into_element()),
            Some(session) => untracked(|| {
                on_session_ready(&session);
                authenticated_shell(session.role).into_element()
            }),
        }))
        .child(toast_host())
}

fn on_session_ready(session: &AccountSession) {
    nav::reset_for_role(session.role);
    enrollment::store::load_roster(session);
}

fn bind_shortcuts() {
    if !platform::is_compact_desktop() && !platform::is_terminal() {
        return;
    }
    on_key("escape", |_| {
        if nav::active_student_no().is_some() {
            nav::pop_detail();
        }
    });
    on_key("1", |_| switch_index(0));
    on_key("2", |_| switch_index(1));
    on_key("3", |_| switch_index(2));
    on_key("4", |_| switch_index(3));
}

fn switch_index(index: usize) {
    let Some(session) = identity_store::current_session() else {
        return;
    };
    let items = nav::primary_destinations(session.role);
    if let Some(dest) = items.get(index).copied() {
        nav::select_destination(dest);
    }
}

fn authenticated_shell(role: Role) -> impl IntoElement {
    if platform::is_terminal() {
        terminal_shell(role).into_element()
    } else if platform::is_mobile() {
        mobile_shell(role).into_element()
    } else {
        desktop_shell(role).into_element()
    }
}

fn desktop_shell(role: Role) -> impl IntoElement {
    let wide = computed(|| window().width >= px(720.0));
    view().flex_1().min_h(px(0.0)).child(dynamic(move || {
        if wide.get() {
            view()
                .flex_1()
                .flex_row()
                .min_h(px(0.0))
                .child(desktop_sidebar(role))
                .child(
                    view()
                        .flex_1()
                        .min_w(px(0.0))
                        .min_h(px(0.0))
                        .child(active_workspace_view()),
                )
                .into_element()
        } else {
            compact_desktop_shell(role).into_element()
        }
    }))
}

fn desktop_sidebar(role: Role) -> impl IntoElement {
    let items = nav::primary_destinations(role);
    let nav_buttons = view().gap_1().children(items.into_iter().map(|dest| {
        sidebar_nav_button(
            dest.label(),
            move || nav::active_student_no().is_none() && nav::current_destination() == dest,
            move || nav::select_destination(dest),
        )
    }));

    let footer = view()
        .gap_2()
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_2()
                .child(typography::sidebar_user_name(move || {
                    identity_store::current_session()
                        .map(|s| SharedString::from(s.display_name))
                        .unwrap_or_default()
                }))
                .child(role_badge(role)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_1()
                .child(app_button(
                    "个人",
                    ButtonVariant::Ghost,
                    ControlSize::Sm,
                    false,
                    || nav::select_destination(NavDestination::Profile),
                ))
                .child(app_button(
                    "退出",
                    ButtonVariant::Ghost,
                    ControlSize::Sm,
                    false,
                    identity_store::sign_out,
                )),
        );

    sidebar_frame("教务工作台", nav_buttons, footer)
}

fn compact_desktop_shell(role: Role) -> impl IntoElement {
    let items = nav::mobile_destinations(role);
    let nav_pills = items.into_iter().map(|dest| {
        compact_nav_pill(
            dest.label(),
            move || nav::active_student_no().is_none() && nav::current_destination() == dest,
            move || nav::select_destination(dest),
        )
    });

    view()
        .flex_1()
        .min_h(px(0.0))
        .child(desktop_window_bar(
            false,
            typography::page_title(move || {
                if nav::active_student_no().is_some() {
                    SharedString::from("学籍详情")
                } else {
                    SharedString::from(nav::current_destination().label())
                }
            }),
            app_button(
                "退出",
                ButtonVariant::Ghost,
                ControlSize::Sm,
                false,
                identity_store::sign_out,
            ),
        ))
        .child(compact_nav_strip(
            view().flex_row().flex_wrap().gap_1().children(nav_pills),
        ))
        .child(active_workspace_view())
}

fn mobile_shell(role: Role) -> impl IntoElement {
    let items = nav::mobile_destinations(role);
    let tab_buttons = items.into_iter().map(|dest| {
        mobile_tab_button(
            dest.label(),
            move || nav::current_destination() == dest,
            move || nav::select_destination(dest),
        )
    });

    let root_body = view()
        .flex_1()
        .min_h(px(0.0))
        .child(active_root_view())
        .child(mobile_tab_strip(
            view().flex_1().flex_row().children(tab_buttons),
        ));

    stacked_shell(
        move || SharedString::from(nav::current_destination().label()),
        root_body,
        nav::detail_stack,
        |no| format!("学籍 · {no}"),
        nav::truncate_detail,
        |no| student_detail_page(&no, identity_store::current_session, nav::pop_detail),
    )
}

fn terminal_shell(role: Role) -> impl IntoElement {
    let items = nav::mobile_destinations(role);
    let nav_rows = view()
        .gap_1()
        .children(items.into_iter().enumerate().map(|(idx, dest)| {
            terminal_nav_row(
                idx,
                dest.label(),
                move || nav::current_destination() == dest,
                move || nav::select_destination(dest),
            )
        }));

    let top_bar = view()
        .flex_1()
        .flex_row()
        .justify_between()
        .items_center()
        .child(typography::body_medium(format!(
            "学生管理系统 · {}",
            role.label()
        )))
        .child(app_button(
            "退出",
            ButtonVariant::Ghost,
            ControlSize::Sm,
            false,
            identity_store::sign_out,
        ));

    terminal_frame(
        top_bar,
        nav_rows,
        active_workspace_view(),
        typography::muted_text("1-4 切换分区  Esc 返回  Tab 焦点"),
    )
}

fn active_workspace_view() -> impl IntoElement {
    dynamic(|| match nav::active_student_no() {
        Some(student_no) => untracked(|| {
            student_detail_page(
                &student_no,
                identity_store::current_session,
                nav::pop_detail,
            )
            .into_element()
        }),
        None => untracked(|| active_root_view().into_element()),
    })
}

fn active_root_view() -> impl IntoElement {
    dynamic(|| {
        let dest = nav::current_destination();
        untracked(|| match dest {
            NavDestination::Students => {
                students_page(identity_store::current_session, |student_no| {
                    nav::push_student_detail(&student_no);
                })
                .into_element()
            }
            NavDestination::Homerooms => {
                homerooms_page(identity_store::current_session).into_element()
            }
            NavDestination::Grading => grading_page(identity_store::current_session).into_element(),
            NavDestination::Accounts => {
                accounts_page(identity_store::current_session).into_element()
            }
            NavDestination::Profile => profile_page(
                identity_store::current_session,
                dark,
                init::toggle_color_scheme,
                identity_store::sign_out,
            )
            .into_element(),
        })
    })
}

use nsh_core::{element::Position, prelude::*};
use student_management_gateway::contract::{AccountSession, Role};

use crate::{
    features::enrollment::{
        cmd_enroll::view::{EnrollDialogState, enroll_dialog, enroll_trigger},
        parts::{student_table_header, student_table_row},
        store::{self, StatusFilter},
    },
    ui::{
        input::controlled_input,
        surface::{desktop_window_bar, empty_state, filter_strip, skeleton_bar},
        tabs::segmented_tabs,
        typography,
    },
};

pub fn students_page(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    on_open_student: impl Fn(String) + Clone + 'static,
) -> impl IntoElement {
    let enroll = EnrollDialogState::new();
    let is_wide = computed(|| window().width >= px(720.0));
    let is_student = move || session().is_some_and(|s| s.role == Role::Student);
    let is_admin = move || session().is_some_and(|s| s.role == Role::Admin);

    view()
        .flex_1()
        .min_h(px(0.0))
        .child(students_top_bar(session, is_wide, is_admin, enroll))
        .child(students_filter_bar(
            session, is_wide, is_student, is_admin, enroll,
        ))
        .child(roster_body(on_open_student, is_wide))
        .child(enroll_dialog(session, enroll))
}

fn students_top_bar(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    is_wide: Memo<bool>,
    is_admin: impl Fn() -> bool + Copy + 'static,
    enroll: EnrollDialogState,
) -> impl IntoElement {
    when(
        move || is_wide.get(),
        move || {
            desktop_window_bar(
                true,
                view()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(typography::page_title("学生"))
                    .child(typography::mono_muted(move || {
                        format!("{} 名", store::student_count())
                    })),
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(view().w(px(200.0)).child(controlled_input(
                        "搜索学号、姓名或班级",
                        store::current_query,
                        move |next| {
                            if let Some(active) = session() {
                                store::set_query(&active, next);
                            }
                        },
                        false,
                    )))
                    .child(when(is_admin, move || enroll_trigger(enroll))),
            )
        },
    )
}

fn students_filter_bar(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    is_wide: Memo<bool>,
    is_student: impl Fn() -> bool + Copy + 'static,
    is_admin: impl Fn() -> bool + Copy + 'static,
    enroll: EnrollDialogState,
) -> impl IntoElement {
    let filter_options = vec![
        (StatusFilter::All, "全部"),
        (StatusFilter::Active, "在籍"),
        (StatusFilter::Suspended, "休学"),
        (StatusFilter::Withdrawn, "退学"),
        (StatusFilter::Graduated, "毕业"),
    ];
    filter_strip(
        view()
            .flex_1()
            .flex_row()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_3()
            .child(when(
                move || !is_student(),
                move || {
                    segmented_tabs(filter_options.clone(), store::current_filter, move |next| {
                        if let Some(active) = session() {
                            store::set_filter(&active, next);
                        }
                    })
                },
            ))
            .child(when(
                move || !is_wide.get(),
                move || {
                    view()
                        .flex_1()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(view().flex_1().min_w(px(160.0)).child(controlled_input(
                            "搜索学号、姓名或班级",
                            store::current_query,
                            move |next| {
                                if let Some(active) = session() {
                                    store::set_query(&active, next);
                                }
                            },
                            false,
                        )))
                        .child(when(is_admin, move || enroll_trigger(enroll)))
                },
            )),
    )
}

fn roster_body(
    on_open_student: impl Fn(String) + Clone + 'static,
    is_wide: Memo<bool>,
) -> impl IntoElement {
    view().flex_1().min_h(px(0.0)).child(dynamic(move || {
        if store::is_loading() {
            return view()
                .p_6()
                .gap_3()
                .child(skeleton_bar(px(240.0), px(16.0)))
                .child(skeleton_bar(px(320.0), px(14.0)))
                .child(skeleton_bar(px(280.0), px(14.0)))
                .into_element();
        }
        if let Some(err) = store::error_message() {
            return view()
                .p_6()
                .child(typography::error_text(err))
                .into_element();
        }
        if store::student_count() == 0 {
            return empty_state("未匹配到学生档案", "可尝试清空搜索词或切换状态筛选。")
                .into_element();
        }
        let show_date_col = is_wide.get();
        let open_cb = on_open_student.clone();
        view()
            .flex_1()
            .min_h(px(0.0))
            .child(student_table_header(show_date_col))
            .child(
                virtual_list(store::student_count, move |at: Position| {
                    let item = computed(move || store::student_at(at.index()));
                    let cb = open_cb.clone();
                    student_table_row(item, show_date_col, cb)
                })
                .keys(store::student_ids)
                .estimated_item_size(px(40.0))
                .flex_1(),
            )
            .into_element()
    }))
}

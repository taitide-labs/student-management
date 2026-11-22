use nsh_core::prelude::*;
use student_management_gateway::contract::{
    AccountSession, EnrollmentChangeRecord, Role, StudentSummary,
};

use crate::{
    features::enrollment::{
        cmd_transition::view::{TransitionModalState, action_card, transition_modal},
        parts::enrollment_status_badge,
        store,
    },
    ui::{
        button::{ButtonVariant, ControlSize, app_button},
        surface::{card, desktop_window_bar, divider, empty_state, section_header},
        typography,
    },
};

pub fn student_detail_page<
    T: Fn() -> Option<AccountSession> + Copy + 'static,
    U: Fn() + Clone + 'static,
>(
    student_no: &str,
    session: T,
    on_back: U,
) -> impl IntoElement + use<T, U> {
    let target = student_no.to_owned();
    let modal = TransitionModalState::new();
    let detail = move || {
        modal.track_refresh();
        session().and_then(|active| store::student_detail(&active, &target).ok())
    };
    let is_wide = computed(|| window().width >= px(720.0));
    let is_admin = move || session().is_some_and(|s| s.role == Role::Admin);
    let student_no_for_modal = student_no.to_owned();

    view()
        .flex_1()
        .min_h(px(0.0))
        .child(when(
            move || is_wide.get(),
            move || {
                let back_cb = on_back.clone();
                desktop_window_bar(
                    true,
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(app_button(
                            "返回",
                            ButtonVariant::Ghost,
                            ControlSize::Sm,
                            false,
                            back_cb,
                        ))
                        .child(typography::page_title("学籍详情")),
                    view(),
                )
            },
        ))
        .child(
            scroll()
                .flex_1()
                .child(view().p_6().gap_4().child(dynamic(move || {
                    match detail() {
                        None => empty_state("无法加载学籍档案", "当前账号无权限查看或记录不存在。")
                            .into_element(),
                        Some(payload) => {
                            detail_sections(&payload.student, &payload.changes, is_admin(), modal)
                                .into_element()
                        }
                    }
                }))),
        )
        .child(transition_modal(session, student_no_for_modal, modal))
}

fn detail_sections(
    student: &StudentSummary,
    changes: &[EnrollmentChangeRecord],
    is_admin: bool,
    modal: TransitionModalState,
) -> impl IntoElement {
    let summary_card = card().child(
        view()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_4()
            .child(
                view()
                    .gap_1()
                    .child(typography::page_title(student.full_name.clone()))
                    .child(typography::mono_muted(format!(
                        "{} · {} · 入学 {}",
                        student.student_no, student.homeroom_name, student.enrolled_on
                    )))
                    .child(typography::body_text(format!(
                        "学籍版本 v{} · 状态变更可追溯",
                        student.version
                    ))),
            )
            .child(enrollment_status_badge(student.status)),
    );

    let change_rows = changes.iter().map(|chg| {
        view()
            .py_2()
            .gap_1()
            .child(typography::body_medium(format!(
                "{} · {} -> {}",
                chg.action, chg.from_summary, chg.to_summary
            )))
            .child(typography::muted_text(format!(
                "{} · {} · {}",
                chg.reason, chg.operator_name, chg.occurred_at
            )))
    });

    let changes_card = card()
        .child(section_header("学籍异动", "不可变学籍变更凭证"))
        .child(divider())
        .children(change_rows);

    let mut root = view().gap_4().child(summary_card);
    if is_admin {
        root = root.child(action_card(student.status, student.version, modal));
    }
    root.child(changes_card)
}

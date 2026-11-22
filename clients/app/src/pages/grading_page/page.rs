use nsh_core::prelude::*;
use student_management_gateway::contract::{
    AccountSession, GradeAmendmentRecord, GradeEntrySummary, GradeStatus, Role,
};

use crate::{
    features::grading::{
        cmd_amend::view::{AmendModalState, amend_modal, amend_trigger},
        cmd_draft::view::{DraftModalState, draft_modal, grading_toolbar},
        store,
    },
    ui::{
        badge::{BadgeTone, badge},
        surface::{card, desktop_window_bar, divider, empty_state, section_header},
        typography,
    },
};

pub fn grading_page(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
) -> impl IntoElement {
    if let Some(active) = untracked(session) {
        store::load_grading(&active);
    }

    let draft_modal_state = DraftModalState::new();
    let amend_modal_state = AmendModalState::new();
    let is_wide = computed(|| window().width >= px(720.0));
    let can_grade =
        move || session().is_some_and(|s| matches!(s.role, Role::Admin | Role::Teacher));
    let is_admin = move || session().is_some_and(|s| s.role == Role::Admin);

    view()
        .flex_1()
        .min_h(px(0.0))
        .child(when(
            move || is_wide.get(),
            move || {
                desktop_window_bar(
                    true,
                    typography::page_title("成绩"),
                    when(can_grade, move || {
                        grading_toolbar(session, draft_modal_state)
                    }),
                )
            },
        ))
        .child(
            scroll().flex_1().child(
                view()
                    .p_6()
                    .gap_4()
                    .child(when(
                        move || !is_wide.get() && can_grade(),
                        move || grading_toolbar(session, draft_modal_state),
                    ))
                    .child(dynamic(move || grading_body(is_admin(), amend_modal_state))),
            ),
        )
        .child(draft_modal(session, draft_modal_state))
        .child(amend_modal(session, amend_modal_state))
}

fn grading_body(is_admin: bool, amend_modal_state: AmendModalState) -> impl IntoElement {
    let entries = store::grades();
    if entries.is_empty() {
        return empty_state("暂无成绩记录", "当前账号名下暂无已录入或已发布成绩。").into_element();
    }
    let grade_rows = entries
        .into_iter()
        .map(|entry| grade_row_card(entry, is_admin, amend_modal_state));
    let mut stack = view().gap_4().children(grade_rows);
    let audits = store::amendments();
    if !audits.is_empty() {
        stack = stack.child(amendments_card(&audits));
    }
    stack.into_element()
}

fn grade_row_card(
    entry: GradeEntrySummary,
    is_admin: bool,
    amend_modal_state: AmendModalState,
) -> impl IntoElement {
    let tone = match entry.status {
        GradeStatus::Draft => BadgeTone::Neutral,
        GradeStatus::Published => BadgeTone::Success,
        GradeStatus::Voided => BadgeTone::Destructive,
    };
    let can_amend = is_admin && entry.status == GradeStatus::Published;
    let grade_id = entry.id;
    let ver = entry.version;
    let score_text = entry.score.normalize().to_string();

    card()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            view()
                .gap_1()
                .child(typography::body_medium(format!(
                    "{} · {}",
                    entry.course_code, entry.course_name
                )))
                .child(typography::mono_muted(format!(
                    "{} · {} · {}",
                    entry.student_no, entry.student_name, entry.term
                ))),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_3()
                .child(typography::mono_text(score_text))
                .child(badge(entry.status.label(), tone))
                .child(when(can_amend, move || {
                    amend_trigger(grade_id.clone(), ver, amend_modal_state)
                })),
        )
}

fn amendments_card(audits: &[GradeAmendmentRecord]) -> impl IntoElement {
    let rows = audits.iter().map(|a| {
        view()
            .py_2()
            .gap_1()
            .child(typography::body_medium(format!(
                "{} · {}（{}）  {} -> {}",
                a.course_name,
                a.student_name,
                a.student_no,
                a.previous_score.normalize(),
                a.new_score.normalize()
            )))
            .child(typography::muted_text(format!(
                "{} · {} · {}",
                a.reason, a.operator_name, a.occurred_at
            )))
    });
    card()
        .child(section_header("合规修订台账", "锁分后成绩更正不可变凭证"))
        .child(divider())
        .children(rows)
}

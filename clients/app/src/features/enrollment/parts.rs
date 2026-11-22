use nsh_core::prelude::*;
use student_management_gateway::contract::{EnrollmentStatus, StudentSummary};

use crate::ui::{
    badge::{BadgeTone, badge},
    table::{table_col_head, table_header_row, table_row_button},
    typography,
};

pub fn enrollment_status_badge(status: EnrollmentStatus) -> impl IntoElement {
    let tone = match status {
        EnrollmentStatus::Active => BadgeTone::Success,
        EnrollmentStatus::Suspended => BadgeTone::Warning,
        EnrollmentStatus::Withdrawn => BadgeTone::Destructive,
        EnrollmentStatus::Graduated => BadgeTone::Primary,
    };
    badge(status.label(), tone)
}

pub fn student_table_header(show_date_col: bool) -> impl IntoElement {
    let compact = !show_date_col;
    let no_w = if compact { px(76.0) } else { px(96.0) };
    let room_w = if compact { px(72.0) } else { px(112.0) };
    let status_w = if compact { px(56.0) } else { px(76.0) };
    let gap = if compact { px(8.0) } else { px(12.0) };

    table_header_row(
        compact,
        view()
            .flex_1()
            .flex_row()
            .items_center()
            .gap(gap)
            .child(view().w(no_w).flex_shrink_0().child(table_col_head("学号")))
            .child(
                view()
                    .flex_1()
                    .min_w(px(64.0))
                    .child(table_col_head("姓名")),
            )
            .child(
                view()
                    .w(room_w)
                    .flex_shrink_0()
                    .child(table_col_head("行政班")),
            )
            .when(show_date_col, |row| {
                row.child(
                    view()
                        .w(px(112.0))
                        .flex_shrink_0()
                        .child(table_col_head("入学日期")),
                )
            })
            .child(
                view()
                    .w(status_w)
                    .flex_shrink_0()
                    .items_end()
                    .child(table_col_head("状态")),
            ),
    )
}

pub fn student_table_row(
    student: Memo<Option<StudentSummary>>,
    show_date_col: bool,
    on_open: impl Fn(String) + 'static,
) -> impl IntoElement {
    let compact = !show_date_col;
    let no_w = if compact { px(76.0) } else { px(96.0) };
    let room_w = if compact { px(72.0) } else { px(112.0) };
    let status_w = if compact { px(56.0) } else { px(76.0) };
    let gap = if compact { px(8.0) } else { px(12.0) };

    let content = view()
        .flex_1()
        .flex_row()
        .items_center()
        .gap(gap)
        .child(
            view()
                .w(no_w)
                .flex_shrink_0()
                .child(typography::mono_muted(move || {
                    student
                        .get()
                        .map(|s| SharedString::from(s.student_no))
                        .unwrap_or_default()
                })),
        )
        .child(
            view()
                .flex_1()
                .min_w(px(64.0))
                .child(typography::body_medium(move || {
                    student
                        .get()
                        .map(|s| SharedString::from(s.full_name))
                        .unwrap_or_default()
                })),
        )
        .child(
            view()
                .w(room_w)
                .flex_shrink_0()
                .child(typography::muted_text(move || {
                    student
                        .get()
                        .map(|s| SharedString::from(s.homeroom_name))
                        .unwrap_or_default()
                })),
        )
        .when(show_date_col, |row| {
            row.child(
                view()
                    .w(px(112.0))
                    .flex_shrink_0()
                    .child(typography::mono_muted(move || {
                        student
                            .get()
                            .map(|s| SharedString::from(s.enrolled_on))
                            .unwrap_or_default()
                    })),
            )
        })
        .child(
            view()
                .w(status_w)
                .flex_shrink_0()
                .items_end()
                .child(dynamic(move || {
                    let status = student.get().map_or(EnrollmentStatus::Active, |s| s.status);
                    enrollment_status_badge(status).into_element()
                })),
        );

    table_row_button(
        compact,
        move || {
            if let Some(st) = student.get_untracked() {
                on_open(st.student_no);
            }
        },
        content,
    )
}

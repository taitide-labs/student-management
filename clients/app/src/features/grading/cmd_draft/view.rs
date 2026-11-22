use nsh_core::prelude::*;
use student_management_gateway::contract::AccountSession;

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button},
    dialog::modal_sheet,
    input::{app_input, labeled_field},
    select::{SelectOption, app_select},
    toast::{ToastTone, push_toast},
};

use super::super::store;
use super::handler;

#[derive(Clone, Copy)]
pub struct DraftModalState {
    open: Signal<bool>,
    teaching: Signal<Option<SharedString>>,
    student_no: Signal<String>,
    score: Signal<String>,
    error: Signal<Option<SharedString>>,
}

impl DraftModalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: signal(false),
            teaching: signal(None),
            student_no: signal(String::new()),
            score: signal(String::new()),
            error: signal(None),
        }
    }

    pub fn open_dialog(self) {
        self.teaching
            .set(store::default_draft_teaching_id().map(SharedString::from));
        self.student_no.set(String::new());
        self.score.set(String::new());
        self.error.set(None);
        self.open.set(true);
    }
}

pub fn grading_toolbar(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    draft_modal: DraftModalState,
) -> impl IntoElement {
    let on_publish = move || {
        let Some(active) = session() else {
            return;
        };
        let Some(target) = store::default_draft_teaching_id() else {
            push_toast("暂无可发布的教学任务。", ToastTone::Error);
            return;
        };
        spawn(async move {
            match handler::publish_teaching(&active, &target).await {
                Ok(count) => push_toast(format!("已锁分发布 {count} 条成绩"), ToastTone::Success),
                Err(err) => push_toast(err, ToastTone::Error),
            }
        });
    };

    view()
        .flex_row()
        .items_center()
        .gap_2()
        .child(app_button(
            "录入成绩…",
            ButtonVariant::Primary,
            ControlSize::Sm,
            false,
            move || draft_modal.open_dialog(),
        ))
        .child(app_button(
            "锁分发布",
            ButtonVariant::Outline,
            ControlSize::Sm,
            false,
            on_publish,
        ))
}

pub fn draft_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    modal: DraftModalState,
) -> impl IntoElement {
    modal_sheet(modal.open, "保存成绩草稿", move || {
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let teaching_id = modal
                .teaching
                .get_untracked()
                .map(|v| v.to_string())
                .unwrap_or_default();
            let student_no = modal.student_no.get_untracked();
            let score_raw = modal.score.get_untracked();
            spawn(async move {
                match handler::save_draft(&active, &teaching_id, &student_no, &score_raw).await {
                    Ok(_) => {
                        modal.open.set(false);
                        push_toast("已保存成绩草稿", ToastTone::Success);
                    }
                    Err(err) => {
                        modal.error.set(Some(err.clone()));
                        push_toast(err, ToastTone::Error);
                    }
                }
            });
        };

        view()
            .gap_3()
            .child(labeled_field(
                "教学任务",
                app_select(
                    || {
                        store::teachings()
                            .into_iter()
                            .map(|t| {
                                SelectOption::new(
                                    t.id,
                                    format!("{} · {}", t.course_code, t.homeroom_name),
                                )
                            })
                            .collect()
                    },
                    modal.teaching,
                    "选择教学任务",
                    "教学任务",
                ),
                || None,
            ))
            .child(labeled_field(
                "学生学号",
                app_input("请输入学生学号", modal.student_no, false),
                || None,
            ))
            .child(labeled_field(
                "分值",
                app_input("请输入分值", modal.score, false),
                move || modal.error.get(),
            ))
            .child(
                view()
                    .flex_row()
                    .justify_end()
                    .gap_2()
                    .child(app_button(
                        "取消",
                        ButtonVariant::Default,
                        ControlSize::Md,
                        false,
                        move || modal.open.set(false),
                    ))
                    .child(app_button(
                        "保存草稿",
                        ButtonVariant::Primary,
                        ControlSize::Md,
                        false,
                        submit,
                    )),
            )
    })
}

use nsh_core::prelude::*;
use student_management_gateway::contract::AccountSession;

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button},
    dialog::modal_sheet,
    input::{app_input, labeled_field},
    toast::{ToastTone, push_toast},
};

use super::handler;

#[derive(Clone, Copy)]
pub struct AmendModalState {
    open: Signal<bool>,
    grade_id: Signal<String>,
    version: Signal<u64>,
    score: Signal<String>,
    reason: Signal<String>,
    error: Signal<Option<SharedString>>,
}

impl AmendModalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: signal(false),
            grade_id: signal(String::new()),
            version: signal(0_u64),
            score: signal(String::new()),
            reason: signal(String::new()),
            error: signal(None),
        }
    }

    pub fn open_for(self, grade_id: String, version: u64) {
        self.grade_id.set(grade_id);
        self.version.set(version);
        self.score.set(String::new());
        self.reason.set(String::new());
        self.error.set(None);
        self.open.set(true);
    }
}

pub fn amend_trigger(grade_id: String, version: u64, modal: AmendModalState) -> impl IntoElement {
    app_button(
        "修订…",
        ButtonVariant::Outline,
        ControlSize::Sm,
        false,
        move || modal.open_for(grade_id.clone(), version),
    )
}

pub fn amend_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    modal: AmendModalState,
) -> impl IntoElement {
    modal_sheet(modal.open, "合规修订成绩", move || {
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let id = modal.grade_id.get_untracked();
            let ver = modal.version.get_untracked();
            let score = modal.score.get_untracked();
            let reason = modal.reason.get_untracked();
            spawn(async move {
                match handler::execute(&active, &id, ver, &score, &reason).await {
                    Ok(_) => {
                        modal.open.set(false);
                        push_toast("已完成成绩合规修订", ToastTone::Success);
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
                "修订后分值",
                app_input("请输入修订后分值", modal.score, false),
                || None,
            ))
            .child(labeled_field(
                "修订原因",
                app_input("请输入修订原因", modal.reason, false),
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
                        "确认修订",
                        ButtonVariant::Primary,
                        ControlSize::Md,
                        false,
                        submit,
                    )),
            )
    })
}

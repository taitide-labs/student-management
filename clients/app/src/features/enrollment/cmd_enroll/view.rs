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
pub struct EnrollDialogState {
    open: Signal<bool>,
    name: Signal<String>,
    room: Signal<Option<SharedString>>,
    error: Signal<Option<SharedString>>,
}

impl EnrollDialogState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: signal(false),
            name: signal(String::new()),
            room: signal(None),
            error: signal(None),
        }
    }

    pub fn open_dialog(self) {
        self.name.set(String::new());
        self.room.set(None);
        self.error.set(None);
        self.open.set(true);
    }
}

pub fn enroll_trigger(state: EnrollDialogState) -> impl IntoElement {
    app_button(
        "办理入学…",
        ButtonVariant::Primary,
        ControlSize::Sm,
        false,
        move || state.open_dialog(),
    )
}

pub fn enroll_dialog(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    state: EnrollDialogState,
) -> impl IntoElement {
    modal_sheet(state.open, "新生入学建档", move || {
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let name = state.name.get_untracked();
            let room = state.room.get_untracked().map(|v| v.to_string());
            spawn(async move {
                match handler::execute(&active, &name, room.as_deref()).await {
                    Ok(created) => {
                        state.open.set(false);
                        push_toast(
                            format!(
                                "已办理入学：{}（{}）",
                                created.full_name, created.student_no
                            ),
                            ToastTone::Success,
                        );
                    }
                    Err(err) => {
                        state.error.set(Some(err.clone()));
                        push_toast(err, ToastTone::Error);
                    }
                }
            });
        };

        view()
            .gap_3()
            .child(labeled_field(
                "姓名",
                app_input("请输入学生姓名", state.name, false),
                || None,
            ))
            .child(labeled_field(
                "行政班",
                app_select(
                    || {
                        store::homerooms()
                            .into_iter()
                            .map(|room| SelectOption::new(room.id, room.name))
                            .collect()
                    },
                    state.room,
                    "选择行政班",
                    "目标行政班",
                ),
                move || state.error.get(),
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
                        move || state.open.set(false),
                    ))
                    .child(app_button(
                        "确认入学",
                        ButtonVariant::Primary,
                        ControlSize::Md,
                        false,
                        submit,
                    )),
            )
    })
}

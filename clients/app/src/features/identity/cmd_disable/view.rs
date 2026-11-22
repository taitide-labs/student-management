use nsh_core::prelude::*;
use student_management_gateway::contract::AccountSession;

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button, app_button_accessible},
    dialog::modal_sheet,
    input::{app_input, labeled_field},
    toast::{ToastTone, push_toast},
};

use super::handler;

#[derive(Clone, Copy)]
pub struct DisableModalState {
    open: Signal<bool>,
    account_id: Signal<String>,
    version: Signal<u64>,
    reason: Signal<String>,
    error: Signal<Option<SharedString>>,
}

impl DisableModalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: signal(false),
            account_id: signal(String::new()),
            version: signal(0_u64),
            reason: signal(String::new()),
            error: signal(None),
        }
    }

    pub fn open_for(self, account_id: String, version: u64) {
        self.account_id.set(account_id);
        self.version.set(version);
        self.reason.set(String::new());
        self.error.set(None);
        self.open.set(true);
    }
}

pub fn disable_trigger(
    account_id: String,
    display_name: &str,
    version: u64,
    modal: DisableModalState,
) -> impl IntoElement + use<> {
    app_button_accessible(
        "停用",
        format!("停用{display_name}"),
        ButtonVariant::Destructive,
        ControlSize::Sm,
        false,
        move || modal.open_for(account_id.clone(), version),
    )
}

pub fn disable_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    modal: DisableModalState,
) -> impl IntoElement {
    modal_sheet(modal.open, "停用账号", move || {
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let id = modal.account_id.get_untracked();
            let ver = modal.version.get_untracked();
            let reason = modal.reason.get_untracked();
            match handler::execute(&active, &id, &reason, ver) {
                Ok(_) => {
                    modal.open.set(false);
                    push_toast("账号已停用", ToastTone::Success);
                }
                Err(err) => {
                    modal.error.set(Some(err.clone()));
                    push_toast(err, ToastTone::Error);
                }
            }
        };

        view()
            .gap_3()
            .child(labeled_field(
                "停用原因",
                app_input("请输入停用原因", modal.reason, false),
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
                        "确认停用",
                        ButtonVariant::Destructive,
                        ControlSize::Md,
                        false,
                        submit,
                    )),
            )
    })
}

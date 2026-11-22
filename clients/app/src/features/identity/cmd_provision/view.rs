use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, Role};

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button},
    dialog::modal_sheet,
    input::{app_input, labeled_field},
    select::{SelectOption, app_select},
    toast::{ToastTone, push_toast},
};

use super::handler;

#[derive(Clone, Copy)]
pub struct ProvisionModalState {
    open: Signal<bool>,
    user: Signal<String>,
    name: Signal<String>,
    role: Signal<Option<SharedString>>,
    student: Signal<String>,
    error: Signal<Option<SharedString>>,
}

impl ProvisionModalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: signal(false),
            user: signal(String::new()),
            name: signal(String::new()),
            role: signal(Some(SharedString::from("teacher"))),
            student: signal(String::new()),
            error: signal(None),
        }
    }

    pub fn open_dialog(self) {
        self.user.set(String::new());
        self.name.set(String::new());
        self.role.set(Some(SharedString::from("teacher")));
        self.student.set(String::new());
        self.error.set(None);
        self.open.set(true);
    }
}

pub fn provision_trigger(modal: ProvisionModalState) -> impl IntoElement {
    app_button(
        "开通账号…",
        ButtonVariant::Primary,
        ControlSize::Sm,
        false,
        move || modal.open_dialog(),
    )
}

pub fn provision_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    modal: ProvisionModalState,
) -> impl IntoElement {
    modal_sheet(modal.open, "开通登录账号", move || {
        let role_opts = vec![
            SelectOption::new("teacher", "教师"),
            SelectOption::new("admin", "教务管理员"),
            SelectOption::new("student", "学生"),
        ];
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let username = modal.user.get_untracked();
            let display_name = modal.name.get_untracked();
            let role = match modal.role.get_untracked().as_deref() {
                Some("admin") => Role::Admin,
                Some("student") => Role::Student,
                _ => Role::Teacher,
            };
            let student_no = modal.student.get_untracked();
            match handler::execute(&active, &username, &display_name, role, &student_no) {
                Ok(created) => {
                    modal.open.set(false);
                    push_toast(
                        format!("已开通账号：{}", created.display_name),
                        ToastTone::Success,
                    );
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
                "用户名",
                app_input("请输入登录用户名", modal.user, false),
                || None,
            ))
            .child(labeled_field(
                "姓名",
                app_input("请输入显示姓名", modal.name, false),
                || None,
            ))
            .child(labeled_field(
                "角色",
                app_select(role_opts, modal.role, "选择角色", "账号角色"),
                || None,
            ))
            .child(labeled_field(
                "绑定学号",
                app_input("学生角色可填学号", modal.student, false),
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
                        "确认开通",
                        ButtonVariant::Primary,
                        ControlSize::Md,
                        false,
                        submit,
                    )),
            )
    })
}

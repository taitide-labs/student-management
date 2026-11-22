use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, HomeroomSummary};

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button, app_button_accessible},
    dialog::modal_sheet,
    input::{app_input, labeled_field},
    toast::{ToastTone, push_toast},
    typography,
};

use super::handler;

#[derive(Clone, Copy)]
pub struct HomeroomModalState {
    create_open: Signal<bool>,
    create_name: Signal<String>,
    create_year: Signal<String>,
    create_cap: Signal<String>,
    create_error: Signal<Option<SharedString>>,
    cap_open: Signal<bool>,
    cap_room_id: Signal<String>,
    cap_room_name: Signal<String>,
    cap_version: Signal<u64>,
    cap_value: Signal<String>,
    cap_error: Signal<Option<SharedString>>,
}

impl HomeroomModalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            create_open: signal(false),
            create_name: signal(String::new()),
            create_year: signal(String::new()),
            create_cap: signal(String::new()),
            create_error: signal(None),
            cap_open: signal(false),
            cap_room_id: signal(String::new()),
            cap_room_name: signal(String::new()),
            cap_version: signal(0_u64),
            cap_value: signal(String::new()),
            cap_error: signal(None),
        }
    }

    pub fn open_create(self) {
        self.create_name.set(String::new());
        self.create_year.set(String::new());
        self.create_cap.set(String::new());
        self.create_error.set(None);
        self.create_open.set(true);
    }

    pub fn open_capacity(self, room: &HomeroomSummary) {
        let cap_summary = room
            .capacity
            .map_or_else(|| "当前不限班额".to_owned(), |c| format!("当前额定 {c} 人"));
        self.cap_room_id.set(room.id.clone());
        self.cap_room_name
            .set(format!("{} · {cap_summary}", room.name));
        self.cap_version.set(room.version);
        self.cap_value.set(String::new());
        self.cap_error.set(None);
        self.cap_open.set(true);
    }
}

pub fn create_trigger(state: HomeroomModalState) -> impl IntoElement {
    app_button(
        "新建行政班…",
        ButtonVariant::Primary,
        ControlSize::Sm,
        false,
        move || state.open_create(),
    )
}

pub fn capacity_trigger(
    room: &HomeroomSummary,
    state: HomeroomModalState,
) -> impl IntoElement + use<> {
    let snapshot = room.clone();
    let label = format!("调整{}容量…", room.name);
    app_button_accessible(
        "调整容量…",
        label,
        ButtonVariant::Outline,
        ControlSize::Sm,
        false,
        move || state.open_capacity(&snapshot),
    )
}

pub fn homeroom_modals(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    state: HomeroomModalState,
) -> impl IntoElement {
    view()
        .child(create_modal(session, state))
        .child(capacity_modal(session, state))
}

fn create_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    modal: HomeroomModalState,
) -> impl IntoElement {
    modal_sheet(modal.create_open, "创建行政班", move || {
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let name = modal.create_name.get_untracked();
            let year = modal.create_year.get_untracked();
            let cap = modal.create_cap.get_untracked();
            match handler::create_homeroom(&active, &name, &year, &cap) {
                Ok(created) => {
                    modal.create_open.set(false);
                    push_toast(
                        format!("已创建行政班：{}", created.name),
                        ToastTone::Success,
                    );
                }
                Err(err) => {
                    modal.create_error.set(Some(err.clone()));
                    push_toast(err, ToastTone::Error);
                }
            }
        };
        view()
            .gap_3()
            .child(labeled_field(
                "班级名称",
                app_input("请输入班级名称", modal.create_name, false),
                || None,
            ))
            .child(labeled_field(
                "入学年份",
                app_input("请输入入学年份", modal.create_year, false),
                || None,
            ))
            .child(labeled_field(
                "额定人数",
                app_input("可选额定人数", modal.create_cap, false),
                move || modal.create_error.get(),
            ))
            .child(modal_actions(
                move || modal.create_open.set(false),
                "确认创建",
                submit,
            ))
    })
}

fn capacity_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    modal: HomeroomModalState,
) -> impl IntoElement {
    modal_sheet(modal.cap_open, "调整额定容量", move || {
        let submit = move || {
            let Some(active) = session() else {
                return;
            };
            let id = modal.cap_room_id.get_untracked();
            let cap = modal.cap_value.get_untracked();
            let ver = modal.cap_version.get_untracked();
            match handler::update_capacity(&active, &id, &cap, ver) {
                Ok(updated) => {
                    modal.cap_open.set(false);
                    push_toast(
                        format!("已更新 {} 额定容量", updated.name),
                        ToastTone::Success,
                    );
                }
                Err(err) => {
                    modal.cap_error.set(Some(err.clone()));
                    push_toast(err, ToastTone::Error);
                }
            }
        };
        view()
            .gap_3()
            .child(typography::muted_text(move || modal.cap_room_name.get()))
            .child(labeled_field(
                "新额定容量",
                app_input("留空表示不限班额", modal.cap_value, false),
                move || modal.cap_error.get(),
            ))
            .child(modal_actions(
                move || modal.cap_open.set(false),
                "确认调整",
                submit,
            ))
    })
}

fn modal_actions(
    on_cancel: impl Fn() + 'static,
    confirm_label: &'static str,
    on_confirm: impl Fn() + 'static,
) -> impl IntoElement {
    view()
        .flex_row()
        .justify_end()
        .gap_2()
        .child(app_button(
            "取消",
            ButtonVariant::Default,
            ControlSize::Md,
            false,
            on_cancel,
        ))
        .child(app_button(
            confirm_label,
            ButtonVariant::Primary,
            ControlSize::Md,
            false,
            on_confirm,
        ))
}

use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, EnrollmentStatus};

use crate::ui::{
    button::{ButtonVariant, ControlSize, app_button},
    dialog::modal_sheet,
    input::{app_input, labeled_field},
    select::{SelectOption, app_select},
    surface::{card, divider, section_header},
    toast::{ToastTone, push_toast},
};

use super::super::store;
use super::command::{self, ActionKind};
use super::handler;

#[derive(Clone, Copy)]
pub struct TransitionModalState {
    open: Signal<bool>,
    kind: Signal<Option<ActionKind>>,
    version: Signal<u64>,
    transfer_room: Signal<Option<SharedString>>,
    reverse_status: Signal<Option<SharedString>>,
    reason: Signal<String>,
    form_error: Signal<Option<SharedString>>,
    refresh_tick: Signal<u64>,
}

impl TransitionModalState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: signal(false),
            kind: signal(None),
            version: signal(0_u64),
            transfer_room: signal(None),
            reverse_status: signal(Some(SharedString::from("Active"))),
            reason: signal(String::new()),
            form_error: signal(None),
            refresh_tick: signal(0_u64),
        }
    }

    pub fn track_refresh(self) {
        let _ = self.refresh_tick.get();
    }

    pub fn open_for(self, kind: ActionKind, version: u64) {
        self.kind.set(Some(kind));
        self.version.set(version);
        self.transfer_room.set(None);
        self.reverse_status.set(Some(SharedString::from("Active")));
        self.reason.set(String::new());
        self.form_error.set(None);
        self.open.set(true);
    }
}

pub fn action_card(
    status: EnrollmentStatus,
    version: u64,
    modal: TransitionModalState,
) -> impl IntoElement {
    let buttons = command::allowed_actions(status).into_iter().map(|kind| {
        let variant = match kind {
            ActionKind::Withdraw => ButtonVariant::Destructive,
            ActionKind::Resume | ActionKind::Graduate => ButtonVariant::Primary,
            ActionKind::Suspend => ButtonVariant::Default,
            ActionKind::Transfer | ActionKind::Reverse => ButtonVariant::Outline,
        };
        app_button(
            kind.button_label(),
            variant,
            ControlSize::Sm,
            false,
            move || modal.open_for(kind, version),
        )
    });
    card()
        .child(section_header("学籍操作", "按状态机约束发起学籍变更"))
        .child(divider())
        .child(view().flex_row().flex_wrap().gap_2().children(buttons))
}

pub fn transition_modal(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    student_no: String,
    modal: TransitionModalState,
) -> impl IntoElement {
    let title = move || {
        SharedString::from(
            modal
                .kind
                .get()
                .map_or("学籍操作", ActionKind::dialog_title),
        )
    };
    modal_sheet(modal.open, title, move || {
        let kind = modal.kind.get().unwrap_or(ActionKind::Suspend);
        let room_options = store::homerooms()
            .into_iter()
            .map(|r| SelectOption::new(r.id, r.name))
            .collect::<Vec<_>>();
        let reverse_options = vec![
            SelectOption::new("Active", "在籍"),
            SelectOption::new("Suspended", "休学"),
        ];
        let target_no = student_no.clone();
        let submit = move || submit_modal(session, &target_no, kind, modal);

        let mut fields = view().gap_3();
        if kind == ActionKind::Transfer {
            fields = fields.child(labeled_field(
                "目标行政班",
                app_select(
                    room_options,
                    modal.transfer_room,
                    "选择目标行政班",
                    "目标行政班",
                ),
                || None,
            ));
        }
        if kind == ActionKind::Reverse {
            fields = fields.child(labeled_field(
                "恢复状态",
                app_select(
                    reverse_options,
                    modal.reverse_status,
                    "选择恢复状态",
                    "恢复状态",
                ),
                || None,
            ));
        }
        fields
            .child(labeled_field(
                "异动原因",
                app_input("请输入异动原因", modal.reason, false),
                move || modal.form_error.get(),
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
                        "确认提交",
                        ButtonVariant::Primary,
                        ControlSize::Md,
                        false,
                        submit,
                    )),
            )
    })
}

fn submit_modal(
    session: impl Fn() -> Option<AccountSession> + Copy,
    student_no: &str,
    kind: ActionKind,
    modal: TransitionModalState,
) {
    let Some(active) = session() else {
        return;
    };
    let room_val = modal.transfer_room.get_untracked().map(|v| v.to_string());
    let rev_val = match modal.reverse_status.get_untracked().as_deref() {
        Some("Suspended") => EnrollmentStatus::Suspended,
        _ => EnrollmentStatus::Active,
    };
    let reason = modal.reason.get_untracked();
    match handler::execute(
        &active,
        student_no,
        modal.version.get_untracked(),
        kind,
        room_val.as_deref(),
        rev_val,
        &reason,
    ) {
        Ok(_) => {
            modal.open.set(false);
            modal.refresh_tick.update(|n| *n += 1);
            push_toast("学籍异动已登记", ToastTone::Success);
        }
        Err(err) => {
            modal.form_error.set(Some(err.clone()));
            push_toast(err, ToastTone::Error);
        }
    }
}

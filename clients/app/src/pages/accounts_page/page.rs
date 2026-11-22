use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, AccountStatus, AccountSummary, Role};

use crate::{
    features::identity::{
        cmd_disable::view::{DisableModalState, disable_modal, disable_trigger},
        cmd_provision::view::{ProvisionModalState, provision_modal, provision_trigger},
        store,
    },
    ui::{
        badge::{BadgeTone, badge},
        button::{ButtonVariant, ControlSize, app_button},
        surface::{card, desktop_window_bar, empty_state},
        toast::{ToastTone, push_toast},
        typography,
    },
};

pub fn accounts_page(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
) -> impl IntoElement {
    if let Some(active) = untracked(session) {
        store::load_accounts(&active);
    }

    let prov_modal = ProvisionModalState::new();
    let dis_modal = DisableModalState::new();
    let is_wide = computed(|| window().width >= px(720.0));
    let is_admin = move || session().is_some_and(|s| s.role == Role::Admin);

    view()
        .flex_1()
        .min_h(px(0.0))
        .child(when(
            move || is_wide.get(),
            move || {
                desktop_window_bar(
                    true,
                    typography::page_title("账号"),
                    when(is_admin, move || provision_trigger(prov_modal)),
                )
            },
        ))
        .child(
            scroll().flex_1().child(
                view()
                    .p_6()
                    .gap_3()
                    .child(when(
                        move || !is_wide.get() && is_admin(),
                        move || provision_trigger(prov_modal),
                    ))
                    .child(dynamic(move || {
                        let list = store::accounts();
                        if list.is_empty() {
                            return empty_state(
                                "无账号管理权限",
                                "请使用教务管理员账号查看并管理登录凭证。",
                            )
                            .into_element();
                        }
                        let current_id = session().map(|s| s.account_id).unwrap_or_default();
                        let rows = list
                            .into_iter()
                            .map(|acc| account_card(acc, &current_id, session, dis_modal));
                        view().gap_3().children(rows).into_element()
                    })),
            ),
        )
        .child(provision_modal(session, prov_modal))
        .child(disable_modal(session, dis_modal))
}

fn account_card(
    acc: AccountSummary,
    current_id: &str,
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
    dis_modal: DisableModalState,
) -> impl IntoElement {
    let status_tone = match acc.status {
        AccountStatus::Active => BadgeTone::Success,
        AccountStatus::Disabled => BadgeTone::Destructive,
    };
    let is_self = acc.account_id == current_id;
    let can_act = !is_self && acc.status == AccountStatus::Active;
    let acc_id = acc.account_id;
    let acc_id_for_role = acc_id.clone();
    let ver = acc.version;
    let name_for_btn = acc.display_name.clone();
    let role_for_cycle = acc.role;
    let meta = acc.bound_student_no.as_deref().map_or_else(
        || format!("@{}", acc.username),
        |no| format!("@{} · 学号 {no}", acc.username),
    );

    card()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            view()
                .gap_1()
                .child(typography::body_medium(acc.display_name))
                .child(typography::mono_muted(meta)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(badge(acc.role.label(), BadgeTone::Neutral))
                .child(badge(acc.status.label(), status_tone))
                .child(when(can_act && acc.role != Role::Student, move || {
                    let id = acc_id_for_role.clone();
                    let next_role = if role_for_cycle == Role::Teacher {
                        Role::Admin
                    } else {
                        Role::Teacher
                    };
                    app_button(
                        "切换角色",
                        ButtonVariant::Outline,
                        ControlSize::Sm,
                        false,
                        move || {
                            if let Some(active) = session() {
                                match store::change_account_role(&active, &id, next_role, ver) {
                                    Ok(updated) => push_toast(
                                        format!(
                                            "已将 {} 调整为{}",
                                            updated.display_name,
                                            updated.role.label()
                                        ),
                                        ToastTone::Success,
                                    ),
                                    Err(err) => push_toast(err, ToastTone::Error),
                                }
                            }
                        },
                    )
                }))
                .child(when(can_act, move || {
                    disable_trigger(acc_id.clone(), &name_for_btn, ver, dis_modal)
                })),
        )
}

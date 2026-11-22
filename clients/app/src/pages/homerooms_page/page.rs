use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, HomeroomSummary, Role};

use crate::{
    features::enrollment::{
        cmd_homeroom::view::{
            HomeroomModalState, capacity_trigger, create_trigger, homeroom_modals,
        },
        store,
    },
    ui::{
        badge::{BadgeTone, badge},
        surface::{card, desktop_window_bar},
        typography,
    },
};

pub fn homerooms_page(
    session: impl Fn() -> Option<AccountSession> + Copy + 'static,
) -> impl IntoElement {
    let modal = HomeroomModalState::new();
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
                    typography::page_title("行政班"),
                    when(is_admin, move || create_trigger(modal)),
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
                        move || create_trigger(modal),
                    ))
                    .child(dynamic(move || {
                        let rows = store::homerooms()
                            .into_iter()
                            .map(|room| homeroom_card(room, is_admin(), modal));
                        view().gap_3().children(rows)
                    })),
            ),
        )
        .child(homeroom_modals(session, modal))
}

fn homeroom_card(
    room: HomeroomSummary,
    is_admin: bool,
    modal: HomeroomModalState,
) -> impl IntoElement {
    let cap_text = room.capacity.map_or_else(
        || format!("在籍 {} 人 · 不限班额", room.active_count),
        |c| format!("在籍 {} / 额定 {} 人", room.active_count, c),
    );
    let room_for_btn = room.clone();

    card()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            view()
                .gap_1()
                .child(typography::body_medium(room.name))
                .child(typography::mono_muted(cap_text)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(badge(format!("{} 级", room.grade_year), BadgeTone::Neutral))
                .child(when(is_admin, move || {
                    capacity_trigger(&room_for_btn, modal)
                })),
        )
}

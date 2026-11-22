use nsh_core::prelude::*;
use student_management_gateway::contract::Role;

use crate::ui::badge::{BadgeTone, badge};

pub fn role_badge(role: Role) -> impl IntoElement {
    let tone = match role {
        Role::Admin => BadgeTone::Primary,
        Role::Teacher => BadgeTone::Success,
        Role::Student => BadgeTone::Neutral,
    };
    badge(role.label(), tone)
}

use nsh_core::prelude::*;
use student_management_gateway::contract::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavDestination {
    #[default]
    Students,
    Homerooms,
    Grading,
    Accounts,
    Profile,
}

impl NavDestination {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Students => "学生",
            Self::Homerooms => "行政班",
            Self::Grading => "成绩",
            Self::Accounts => "账号",
            Self::Profile => "个人",
        }
    }
}

#[derive(Clone, Default)]
struct NavState {
    destination: NavDestination,
    detail_stack: Vec<String>,
}

fn state() -> Signal<NavState> {
    global::<NavState>()
}

#[must_use]
pub fn current_destination() -> NavDestination {
    state().with(|s| s.destination)
}

#[must_use]
pub fn detail_stack() -> Vec<String> {
    state().with(|s| s.detail_stack.clone())
}

#[must_use]
pub fn active_student_no() -> Option<String> {
    state().with(|s| s.detail_stack.last().cloned())
}

pub fn select_destination(dest: NavDestination) {
    state().update(|s| {
        s.destination = dest;
        s.detail_stack.clear();
    });
}

pub fn push_student_detail(student_no: &str) {
    state().update(|s| {
        if s.detail_stack.last().map(String::as_str) != Some(student_no) {
            s.detail_stack.push(student_no.to_owned());
        }
    });
}

pub fn pop_detail() {
    state().update(|s| {
        s.detail_stack.pop();
    });
}

pub fn truncate_detail(depth: usize) {
    state().update(|s| s.detail_stack.truncate(depth));
}

pub fn reset_for_role(role: Role) {
    let default_dest = match role {
        Role::Admin | Role::Teacher => NavDestination::Students,
        Role::Student => NavDestination::Profile,
    };
    state().update(|s| {
        s.destination = default_dest;
        s.detail_stack.clear();
    });
}

#[must_use]
pub fn primary_destinations(role: Role) -> Vec<NavDestination> {
    match role {
        Role::Admin => vec![
            NavDestination::Students,
            NavDestination::Homerooms,
            NavDestination::Grading,
            NavDestination::Accounts,
        ],
        Role::Teacher => vec![NavDestination::Students, NavDestination::Grading],
        Role::Student => vec![NavDestination::Profile, NavDestination::Students],
    }
}

#[must_use]
pub fn mobile_destinations(role: Role) -> Vec<NavDestination> {
    let mut items = primary_destinations(role);
    if !items.contains(&NavDestination::Profile) {
        items.push(NavDestination::Profile);
    }
    items
}

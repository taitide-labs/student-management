use nsh_core::prelude::*;
use student_management_gateway::contract::{AccountSession, Role};

use crate::{
    features::identity::store,
    ui::{
        button::{ButtonVariant, ControlSize, app_button},
        input::{app_input, labeled_field},
        surface::{card, divider, section_header},
        typography,
    },
};

pub fn login_page(on_signed_in: impl Fn(AccountSession) + Clone + 'static) -> impl IntoElement {
    let username = signal(String::from("admin"));
    let password = signal(String::from("demo"));

    let submit_cb = on_signed_in.clone();
    let submit = move || {
        if let Ok(session) = store::sign_in(&username.get_untracked(), &password.get_untracked()) {
            submit_cb(session);
        }
    };

    let quick_btn = move |role: Role, label: &'static str, cb: Box<dyn Fn(AccountSession)>| {
        app_button(
            label,
            ButtonVariant::Outline,
            ControlSize::Sm,
            false,
            move || {
                if let Ok(session) = store::quick_sign_in(role) {
                    cb(session);
                }
            },
        )
    };

    view().flex_1().items_center().justify_center().p_6().child(
        card()
            .w_full()
            .max_w(px(360.0))
            .p_6()
            .gap_4()
            .child(section_header("学生管理系统", "教务与学籍工作台"))
            .child(divider())
            .child(labeled_field(
                "账号",
                app_input("admin / teacher / student", username, false),
                || None,
            ))
            .child(labeled_field(
                "密码",
                app_input("输入登录密码", password, true),
                store::auth_error,
            ))
            .child(app_button(
                "登录",
                ButtonVariant::Primary,
                ControlSize::Lg,
                move || username.with(|u| u.trim().is_empty()),
                submit,
            ))
            .child(divider())
            .child(
                view()
                    .gap_2()
                    .child(typography::muted_text("演示账号"))
                    .child(
                        view()
                            .flex_row()
                            .flex_wrap()
                            .gap_2()
                            .child(quick_btn(
                                Role::Admin,
                                "管理员",
                                Box::new(on_signed_in.clone()),
                            ))
                            .child(quick_btn(
                                Role::Teacher,
                                "教师",
                                Box::new(on_signed_in.clone()),
                            ))
                            .child(quick_btn(Role::Student, "学生", Box::new(on_signed_in))),
                    ),
            ),
    )
}

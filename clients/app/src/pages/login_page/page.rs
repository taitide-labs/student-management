use nsh_core::prelude::*;

use crate::{
    features::identity::store,
    ui::{
        button::{ButtonVariant, ControlSize, app_button},
        input::{app_input, labeled_field},
        surface::{card, divider, section_header},
    },
};

pub fn login_page() -> impl IntoElement {
    let username = signal(String::new());
    let password = signal(String::new());

    let submit = move || {
        let user = username.get_untracked();
        let pass = password.get_untracked();
        spawn(async move {
            store::sign_in(&user, &pass).await;
        });
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
                app_input("请输入登录账号", username, false),
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
            )),
    )
}

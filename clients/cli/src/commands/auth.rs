use clap::{Arg, ArgMatches, Command};
use student_management_gateway::contract::{AccountSession, Gateway, SignInRequest};

use super::shared::{emit_error, emit_json_or_text, str_arg, usage_err};

#[must_use]
pub fn command() -> Command {
    Command::new("auth")
        .about("管理登录会话与认证令牌")
        .subcommand_required(true)
        .subcommand(
            Command::new("login")
                .about("登录获取访问令牌")
                .arg(Arg::new("username").long("username").required(true))
                .arg(Arg::new("password").long("password").required(true)),
        )
        .subcommand(Command::new("whoami").about("查看当前登录会话"))
        .subcommand(Command::new("logout").about("清除本地会话令牌"))
}

pub async fn execute<G: Gateway>(
    gw: &G,
    session: Option<&AccountSession>,
    sub: &ArgMatches,
    json: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("login", args)) => {
            let req = SignInRequest {
                username: str_arg(args, "username").to_owned(),
                password: str_arg(args, "password").to_owned(),
            };
            match gw.sign_in(&req).await {
                Ok(signed) => emit_json_or_text(&signed, json, &signed.access_token),
                Err(err) => emit_error(&err),
            }
        }
        Some(("whoami", _)) => match session {
            Some(active) => emit_json_or_text(active, json, &active.username),
            None => usage_err("当前未提供有效会话或令牌。"),
        },
        Some(("logout", _)) => {
            emit_json_or_text(&serde_json::json!({ "ok": true }), json, "已登出")
        }
        _ => 2,
    }
}

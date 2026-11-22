use std::io::{Write, stderr};

use clap::ArgMatches;
use student_management_gateway::{
    contract::{AccountSession, Gateway, Role, SignInRequest},
    local::LocalGatewayHandle,
    remote::{RemoteGateway, RemoteGatewayConfig},
};

use crate::commands::{
    self,
    shared::{emit_error, usage_err},
};

fn dispatch_command<G: Gateway>(
    gw: &mut G,
    session: &AccountSession,
    matches: &ArgMatches,
    json: bool,
    yes: bool,
) -> u8 {
    match matches.subcommand() {
        Some(("account", sub)) => commands::account::execute(gw, session, sub, json, yes),
        Some(("homeroom", sub)) => commands::homeroom::execute(gw, session, sub, json),
        Some(("student", sub)) => commands::student::execute(gw, session, sub, json, yes),
        Some(("course", sub)) => commands::grading::execute_course(gw, session, sub, json),
        Some(("teaching", sub)) => commands::grading::execute_teaching(gw, session, sub, json),
        Some(("grade", sub)) => commands::grading::execute_grade(gw, session, sub, json),
        _ => {
            let _err = writeln!(
                stderr(),
                "error [not_implemented]: 该子命令将在后续切片实现。"
            );
            1
        }
    }
}

fn run_local(matches: &ArgMatches, json: bool, yes: bool) -> u8 {
    let db_url = matches
        .get_one::<String>("db")
        .map_or("sqlite::memory:", String::as_str);

    let mut gw = match LocalGatewayHandle::open(db_url) {
        Ok(handle) => handle,
        Err(err) => return emit_error(&err),
    };
    if let Err(err) = gw.ensure_bootstrap_admin("admin", "系统管理员", "admin") {
        return emit_error(&err);
    }

    let token = matches.get_one::<String>("token").map(String::as_str);
    if let Some(("auth", sub)) = matches.subcommand() {
        let active = match token {
            Some(tok) => gw.resolve_session(tok).ok(),
            None => gw
                .sign_in(&SignInRequest {
                    username: "admin".to_owned(),
                    password: "admin".to_owned(),
                })
                .ok(),
        };
        return commands::auth::execute(&gw, active.as_ref(), sub, json);
    }

    let session = match token {
        Some(tok) => gw.resolve_session(tok),
        None => gw.sign_in(&SignInRequest {
            username: "admin".to_owned(),
            password: "admin".to_owned(),
        }),
    };
    let session = match session {
        Ok(s) => s,
        Err(err) => return emit_error(&err),
    };

    dispatch_command(&mut gw, &session, matches, json, yes)
}

fn run_remote(server: &str, matches: &ArgMatches, json: bool, yes: bool) -> u8 {
    let mut gw = RemoteGateway::new(&RemoteGatewayConfig {
        base_url: server.to_owned(),
    });
    let token = matches.get_one::<String>("token").map(String::as_str);
    let token_session = token.map(|tok| AccountSession {
        account_id: String::new(),
        username: String::new(),
        display_name: String::new(),
        role: Role::Admin,
        bound_student_no: None,
        access_token: tok.to_owned(),
    });

    if let Some(("auth", sub)) = matches.subcommand() {
        return commands::auth::execute(&gw, token_session.as_ref(), sub, json);
    }

    let Some(session) = token_session else {
        return usage_err(
            "远程模式访问业务命令请传入 `--token <TOKEN>` 或先执行 `sms --server <URL> auth login`。",
        );
    };

    dispatch_command(&mut gw, &session, matches, json, yes)
}

pub fn run(matches: &ArgMatches) -> u8 {
    let local = matches.get_flag("local");
    let json = matches.get_flag("json");
    let yes = matches.get_flag("yes");

    if local {
        return run_local(matches, json, yes);
    }

    if let Some(server) = matches.get_one::<String>("server") {
        return run_remote(server, matches, json, yes);
    }

    let _err = writeln!(
        stderr(),
        "error [config_missing]: 请传入 --local 或 --server <URL> 指定运行目标。"
    );
    2
}

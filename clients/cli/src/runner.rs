use std::io::{Write, stderr};

use clap::ArgMatches;
use student_management_gateway::{
    contract::{AccountSession, Gateway, GatewayError, Role},
    local::LocalGatewayHandle,
    remote::{RemoteGateway, RemoteGatewayConfig},
};

use crate::commands::{
    self,
    shared::{emit_error, usage_err},
};

async fn dispatch_command<G: Gateway>(
    gw: &G,
    session: &AccountSession,
    matches: &ArgMatches,
    json: bool,
    yes: bool,
) -> u8 {
    match matches.subcommand() {
        Some(("account", sub)) => commands::account::execute(gw, session, sub, json, yes).await,
        Some(("homeroom", sub)) => commands::homeroom::execute(gw, session, sub, json).await,
        Some(("student", sub)) => commands::student::execute(gw, session, sub, json, yes).await,
        Some(("course", sub)) => commands::grading::execute_course(gw, session, sub, json).await,
        Some(("teaching", sub)) => {
            commands::grading::execute_teaching(gw, session, sub, json).await
        }
        Some(("grade", sub)) => commands::grading::execute_grade(gw, session, sub, json).await,
        _ => {
            let _err = writeln!(
                stderr(),
                "error [not_implemented]: 该子命令将在后续切片实现。"
            );
            1
        }
    }
}

async fn run_local(matches: &ArgMatches, json: bool, yes: bool) -> u8 {
    let db_url = matches
        .get_one::<String>("db")
        .map_or("sqlite::memory:", String::as_str);

    let gw = match LocalGatewayHandle::open(db_url).await {
        Ok(handle) => handle,
        Err(err) => return emit_error(&err),
    };

    let token = matches.get_one::<String>("token").map(String::as_str);
    if let Some(("auth", sub)) = matches.subcommand() {
        let active = match token {
            Some(tok) => gw.resolve_session(tok).await.ok(),
            None if sub.subcommand_name() == Some("login") => None,
            None => gw.open_local_admin_session().await.ok(),
        };
        return commands::auth::execute(&gw, active.as_ref(), sub, json).await;
    }

    let session = match token {
        Some(tok) => gw.resolve_session(tok).await,
        None => gw.open_local_admin_session().await,
    };
    let session = match session {
        Ok(s) => s,
        Err(err) => return emit_error(&err),
    };

    dispatch_command(&gw, &session, matches, json, yes).await
}

async fn run_remote(server: &str, matches: &ArgMatches, json: bool, yes: bool) -> u8 {
    let gw = RemoteGateway::new(&RemoteGatewayConfig {
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
        return commands::auth::execute(&gw, token_session.as_ref(), sub, json).await;
    }

    let Some(session) = token_session else {
        return usage_err(
            "远程模式访问业务命令请传入 `--token <TOKEN>` 或先执行 `sms --server <URL> auth login`。",
        );
    };

    dispatch_command(&gw, &session, matches, json, yes).await
}

pub fn run(matches: &ArgMatches) -> u8 {
    let local = matches.get_flag("local");
    let json = matches.get_flag("json");
    let yes = matches.get_flag("yes");

    if !local && matches.get_one::<String>("server").is_none() {
        let _err = writeln!(
            stderr(),
            "error [config_missing]: 请传入 --local 或 --server <URL> 指定运行目标。"
        );
        return 2;
    }

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => return emit_error(&GatewayError::Transport(err.to_string())),
    };

    if local {
        return rt.block_on(run_local(matches, json, yes));
    }

    if let Some(server) = matches.get_one::<String>("server") {
        return rt.block_on(run_remote(server, matches, json, yes));
    }

    2
}

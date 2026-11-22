use std::io::{Write, stderr};

use clap::ArgMatches;
use student_management_gateway::{
    contract::{Gateway, SignInRequest},
    local::LocalGatewayHandle,
};

use crate::commands::{self, shared::emit_error};

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

    match matches.subcommand() {
        Some(("account", sub)) => commands::account::execute(&mut gw, &session, sub, json, yes),
        Some(("homeroom", sub)) => commands::homeroom::execute(&mut gw, &session, sub, json),
        Some(("student", sub)) => commands::student::execute(&mut gw, &session, sub, json, yes),
        Some(("course", sub)) => commands::grading::execute_course(&mut gw, &session, sub, json),
        Some(("teaching", sub)) => {
            commands::grading::execute_teaching(&mut gw, &session, sub, json)
        }
        Some(("grade", sub)) => commands::grading::execute_grade(&mut gw, &session, sub, json),
        _ => {
            let _err = writeln!(
                stderr(),
                "error [not_implemented]: 该子命令将在后续切片实现。"
            );
            1
        }
    }
}

pub fn run(matches: &ArgMatches) -> u8 {
    let local = matches.get_flag("local");
    let json = matches.get_flag("json");
    let yes = matches.get_flag("yes");

    if local {
        return run_local(matches, json, yes);
    }

    if matches.get_one::<String>("server").is_none() {
        let _err = writeln!(
            stderr(),
            "error [config_missing]: 请传入 --local 或 --server <URL> 指定运行目标。"
        );
        return 2;
    }

    let _err = writeln!(
        stderr(),
        "error [not_implemented]: 远程网关将在下一步接入。"
    );
    1
}

use std::io::{Write, stderr};

use clap::ArgMatches;
use student_management_gateway::{
    contract::{Gateway, Role},
    mock::MockGateway,
};

use crate::commands::{self, shared::emit_error};

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

    let mut gw = MockGateway::seeded();
    let session = match gw.switch_role(Role::Admin) {
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

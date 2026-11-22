use std::io::{IsTerminal, Write, stderr, stdin, stdout};

use clap::ArgMatches;
use student_management_gateway::contract::{EnrollmentStatus, GatewayError, Role};

pub fn str_arg<'a>(args: &'a ArgMatches, key: &str) -> &'a str {
    args.get_one::<String>(key).map_or("", String::as_str)
}

pub fn parse_u64(args: &ArgMatches, key: &str) -> u64 {
    str_arg(args, key).parse::<u64>().unwrap_or(0)
}

pub fn require_yes(yes: bool, op: &str) -> Option<u8> {
    if !yes && !stdin().is_terminal() {
        let _err = writeln!(
            stderr(),
            "error: 非交互模式下执行不可逆操作 `{op}` 必须显式传入 `--yes` 确认。"
        );
        return Some(2);
    }
    None
}

pub fn usage_err(msg: &str) -> u8 {
    let _err = writeln!(stderr(), "error [validation]: {msg}");
    2
}

pub fn parse_role(raw: &str) -> Option<Role> {
    match raw {
        "admin" => Some(Role::Admin),
        "teacher" => Some(Role::Teacher),
        "student" => Some(Role::Student),
        _ => None,
    }
}

pub fn parse_status(raw: &str) -> Option<EnrollmentStatus> {
    match raw {
        "active" => Some(EnrollmentStatus::Active),
        "suspended" => Some(EnrollmentStatus::Suspended),
        "withdrawn" => Some(EnrollmentStatus::Withdrawn),
        "graduated" => Some(EnrollmentStatus::Graduated),
        _ => None,
    }
}

pub fn emit_json_or_text<T: serde::Serialize>(value: &T, json: bool, text: &str) -> u8 {
    let mut out = stdout().lock();
    if json {
        match serde_json::to_string_pretty(value) {
            Ok(serialized) => {
                if writeln!(out, "{serialized}").is_err() {
                    return 1;
                }
            }
            Err(_) => return 1,
        }
    } else if writeln!(out, "{text}").is_err() {
        return 1;
    }
    0
}

pub fn emit_error(err: &GatewayError) -> u8 {
    let mut err_out = stderr().lock();
    match err {
        GatewayError::Problem(p) => {
            let _w = writeln!(
                err_out,
                "error [{}] ({}): {} — {}",
                p.code, p.status, p.title, p.detail
            );
            match p.status {
                401 | 403 => 3,
                404 | 409 | 422 => 4,
                _ => 1,
            }
        }
        GatewayError::Transport(msg) => {
            let _w = writeln!(err_out, "error [transport]: {msg}");
            1
        }
    }
}

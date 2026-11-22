use std::io::{Write, stdout};

use clap::{Arg, ArgMatches, Command};
use student_management_gateway::contract::{
    AccountSession, EnrollStudentRequest, Gateway, ReverseEnrollmentRequest, StudentSummary,
    TransferHomeroomRequest, TransitionStudentRequest,
};

use super::shared::{
    emit_error, emit_json_or_text, make_idempotency_key, parse_status, parse_u64, require_yes,
    str_arg, usage_err,
};

#[must_use]
pub fn command() -> Command {
    Command::new("student")
        .about("管理学生档案与学籍异动")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("查询学生名录")
                .arg(Arg::new("q").long("q"))
                .arg(Arg::new("status").long("status")),
        )
        .subcommand(
            Command::new("show")
                .about("查看学生档案与异动记录")
                .arg(Arg::new("student_no").required(true)),
        )
        .subcommand(
            Command::new("enroll")
                .about("办理新生入学建档并分配学号")
                .arg(Arg::new("name").long("name").required(true))
                .arg(Arg::new("homeroom").long("homeroom").required(true)),
        )
        .subcommand(
            Command::new("transfer")
                .about("登记行政转班")
                .arg(Arg::new("student_no").required(true))
                .arg(Arg::new("homeroom").long("homeroom").required(true))
                .arg(Arg::new("reason").long("reason").required(true))
                .arg(Arg::new("version").long("version").required(true)),
        )
        .subcommand(transition_subcommand("suspend", "登记休学"))
        .subcommand(transition_subcommand("resume", "登记复学"))
        .subcommand(transition_subcommand("withdraw", "登记退学"))
        .subcommand(transition_subcommand("graduate", "登记毕业离校"))
        .subcommand(
            Command::new("reverse")
                .about("终态学籍误操作冲正")
                .arg(Arg::new("student_no").required(true))
                .arg(
                    Arg::new("target_status")
                        .long("target-status")
                        .required(true),
                )
                .arg(Arg::new("reason").long("reason").required(true))
                .arg(Arg::new("version").long("version").required(true)),
        )
}

fn transition_subcommand(name: &'static str, about: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .arg(Arg::new("student_no").required(true))
        .arg(Arg::new("reason").long("reason").required(true))
        .arg(Arg::new("version").long("version").required(true))
}

pub fn execute<G: Gateway>(
    gw: &mut G,
    session: &AccountSession,
    sub: &ArgMatches,
    json: bool,
    yes: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("list", args)) => {
            let q = args.get_one::<String>("q").map_or("", String::as_str);
            let filter = args
                .get_one::<String>("status")
                .and_then(|s| parse_status(s));
            match gw.list_students(session, q, filter) {
                Ok(rows) => emit_students(&rows, json),
                Err(err) => emit_error(&err),
            }
        }
        Some(("show", args)) => {
            let no = str_arg(args, "student_no");
            match gw.student_detail(session, no) {
                Ok(detail) => emit_json_or_text(&detail, json, &detail.student.full_name),
                Err(err) => emit_error(&err),
            }
        }
        Some(("enroll", args)) => {
            let name = str_arg(args, "name");
            let homeroom = str_arg(args, "homeroom");
            let req = EnrollStudentRequest {
                full_name: name.to_owned(),
                homeroom_id: homeroom.to_owned(),
                idempotency_key: make_idempotency_key("cli-enroll", &format!("{name}-{homeroom}")),
            };
            match gw.enroll_student(session, &req) {
                Ok(created) => emit_json_or_text(
                    &created,
                    json,
                    &format!(
                        "{}  {}  {}",
                        created.student_no, created.full_name, created.homeroom_name
                    ),
                ),
                Err(err) => emit_error(&err),
            }
        }
        Some(("transfer", args)) => {
            let no = str_arg(args, "student_no");
            let version = parse_u64(args, "version");
            let req = TransferHomeroomRequest {
                target_homeroom_id: str_arg(args, "homeroom").to_owned(),
                reason: str_arg(args, "reason").to_owned(),
                version,
                idempotency_key: format!("cli-transfer-{no}-{version}"),
            };
            match gw.transfer_homeroom(session, no, &req) {
                Ok(updated) => emit_json_or_text(&updated, json, &updated.student_no),
                Err(err) => emit_error(&err),
            }
        }
        Some((cmd @ ("suspend" | "resume" | "withdraw" | "graduate"), args)) => {
            if matches!(cmd, "withdraw" | "graduate")
                && let Some(code) = require_yes(yes, cmd)
            {
                return code;
            }
            let no = str_arg(args, "student_no");
            let version = parse_u64(args, "version");
            let req = TransitionStudentRequest {
                reason: str_arg(args, "reason").to_owned(),
                version,
                idempotency_key: format!("cli-{cmd}-{no}-{version}"),
            };
            let result = match cmd {
                "suspend" => gw.suspend_student(session, no, &req),
                "resume" => gw.resume_student(session, no, &req),
                "withdraw" => gw.withdraw_student(session, no, &req),
                _ => gw.graduate_student(session, no, &req),
            };
            match result {
                Ok(updated) => emit_json_or_text(&updated, json, &updated.student_no),
                Err(err) => emit_error(&err),
            }
        }
        Some(("reverse", args)) => {
            let no = str_arg(args, "student_no");
            let Some(target) = parse_status(str_arg(args, "target_status")) else {
                return usage_err("无效目标状态。");
            };
            let version = parse_u64(args, "version");
            let req = ReverseEnrollmentRequest {
                target_status: target,
                reason: str_arg(args, "reason").to_owned(),
                version,
                idempotency_key: format!("cli-reverse-{no}-{version}"),
            };
            match gw.reverse_enrollment(session, no, &req) {
                Ok(updated) => emit_json_or_text(&updated, json, &updated.student_no),
                Err(err) => emit_error(&err),
            }
        }
        _ => 2,
    }
}

fn emit_students(rows: &[StudentSummary], json: bool) -> u8 {
    if json {
        return emit_json_or_text(&rows, true, "");
    }
    let mut out = stdout().lock();
    if writeln!(
        out,
        "STUDENT_NO  FULL_NAME  HOMEROOM    STATUS  ENROLLED_ON"
    )
    .is_err()
    {
        return 1;
    }
    for row in rows {
        if writeln!(
            out,
            "{}    {}     {}   {}  {}",
            row.student_no,
            row.full_name,
            row.homeroom_name,
            row.status.label(),
            row.enrolled_on
        )
        .is_err()
        {
            return 1;
        }
    }
    0
}

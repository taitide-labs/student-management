use clap::{Arg, ArgMatches, Command};
use rust_decimal::Decimal;
use student_management_gateway::{
    contract::{
        AccountSession, AmendGradeRequest, AssignTeachingRequest, CreateCourseRequest, Gateway,
        PublishGradesRequest, SaveDraftGradeRequest,
    },
    mock::MockGateway,
};

use super::shared::{emit_error, emit_json_or_text, parse_u64, str_arg, usage_err};

#[must_use]
pub fn course_command() -> Command {
    Command::new("course")
        .about("管理课程定义")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("查询课程列表"))
        .subcommand(
            Command::new("create")
                .about("创建课程")
                .arg(Arg::new("code").long("code").required(true))
                .arg(Arg::new("name").long("name").required(true))
                .arg(Arg::new("max_score").long("max-score").required(true))
                .arg(Arg::new("pass_score").long("pass-score").required(true)),
        )
}

#[must_use]
pub fn teaching_command() -> Command {
    Command::new("teaching")
        .about("管理学期教学任务分派")
        .subcommand_required(true)
        .subcommand(Command::new("list").about("查询教学任务"))
        .subcommand(
            Command::new("assign")
                .about("分派课程教学任务")
                .arg(Arg::new("course").long("course").required(true))
                .arg(Arg::new("homeroom").long("homeroom").required(true))
                .arg(Arg::new("term").long("term").required(true))
                .arg(Arg::new("teacher").long("teacher").required(true)),
        )
}

#[must_use]
pub fn grade_command() -> Command {
    Command::new("grade")
        .about("管理成绩草稿录入、锁分发布与修订")
        .subcommand_required(true)
        .subcommand(
            Command::new("list")
                .about("查询课程成绩条目")
                .arg(Arg::new("student_no").long("student-no")),
        )
        .subcommand(
            Command::new("save")
                .about("录入或更新成绩草稿")
                .arg(Arg::new("teaching").long("teaching").required(true))
                .arg(Arg::new("student_no").long("student-no").required(true))
                .arg(Arg::new("score").long("score").required(true)),
        )
        .subcommand(
            Command::new("publish")
                .about("锁定并发布教学任务下全部成绩草稿")
                .arg(Arg::new("teaching").long("teaching").required(true)),
        )
        .subcommand(
            Command::new("amend")
                .about("合规修订已锁定成绩")
                .arg(Arg::new("grade_id").required(true))
                .arg(Arg::new("score").long("score").required(true))
                .arg(Arg::new("reason").long("reason").required(true))
                .arg(Arg::new("version").long("version").required(true)),
        )
        .subcommand(
            Command::new("amendments")
                .about("查看成绩修订台账")
                .arg(Arg::new("grade_id").long("grade-id")),
        )
}

pub fn execute_course(
    gw: &mut MockGateway,
    session: &AccountSession,
    sub: &ArgMatches,
    json: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("list", _)) => match gw.list_courses(session) {
            Ok(rows) => emit_json_or_text(&rows, json, &format!("共 {} 门课程", rows.len())),
            Err(err) => emit_error(&err),
        },
        Some(("create", args)) => {
            let code = str_arg(args, "code");
            let Ok(max_score) = str_arg(args, "max_score").parse::<Decimal>() else {
                return usage_err("max-score 必须是合法数字。");
            };
            let Ok(pass_score) = str_arg(args, "pass_score").parse::<Decimal>() else {
                return usage_err("pass-score 必须是合法数字。");
            };
            let req = CreateCourseRequest {
                code: code.to_owned(),
                name: str_arg(args, "name").to_owned(),
                max_score,
                pass_score,
                idempotency_key: format!("cli-crs-{code}"),
            };
            match gw.create_course(session, &req) {
                Ok(created) => emit_json_or_text(&created, json, &created.code),
                Err(err) => emit_error(&err),
            }
        }
        _ => 2,
    }
}

pub fn execute_teaching(
    gw: &mut MockGateway,
    session: &AccountSession,
    sub: &ArgMatches,
    json: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("list", _)) => match gw.list_teachings(session) {
            Ok(rows) => emit_json_or_text(&rows, json, &format!("共 {} 项教学任务", rows.len())),
            Err(err) => emit_error(&err),
        },
        Some(("assign", args)) => {
            let course_id = str_arg(args, "course").to_owned();
            let homeroom_id = str_arg(args, "homeroom").to_owned();
            let term = str_arg(args, "term").to_owned();
            let teacher_account_id = str_arg(args, "teacher").to_owned();
            let req = AssignTeachingRequest {
                idempotency_key: format!("cli-tch-{course_id}-{homeroom_id}-{term}"),
                course_id,
                homeroom_id,
                term,
                teacher_account_id,
            };
            match gw.assign_teaching(session, &req) {
                Ok(created) => emit_json_or_text(&created, json, &created.id),
                Err(err) => emit_error(&err),
            }
        }
        _ => 2,
    }
}

pub fn execute_grade(
    gw: &mut MockGateway,
    session: &AccountSession,
    sub: &ArgMatches,
    json: bool,
) -> u8 {
    match sub.subcommand() {
        Some(("list", args)) => {
            let no = args.get_one::<String>("student_no").map(String::as_str);
            match gw.list_grades(session, no) {
                Ok(grades) => {
                    emit_json_or_text(&grades, json, &format!("共 {} 条成绩", grades.len()))
                }
                Err(err) => emit_error(&err),
            }
        }
        Some(("save", args)) => {
            let teaching_id = str_arg(args, "teaching").to_owned();
            let student_no = str_arg(args, "student_no").to_owned();
            let Ok(score) = str_arg(args, "score").parse::<Decimal>() else {
                return usage_err("score 必须是合法数字。");
            };
            let req = SaveDraftGradeRequest {
                idempotency_key: format!("cli-grd-save-{teaching_id}-{student_no}-{score}"),
                teaching_id,
                student_no,
                score,
            };
            match gw.save_draft_grade(session, &req) {
                Ok(saved) => emit_json_or_text(&saved, json, &saved.score.to_string()),
                Err(err) => emit_error(&err),
            }
        }
        Some(("publish", args)) => {
            let teaching_id = str_arg(args, "teaching").to_owned();
            let req = PublishGradesRequest {
                idempotency_key: format!("cli-grd-pub-{teaching_id}"),
                teaching_id,
            };
            match gw.publish_grades(session, &req) {
                Ok(rows) => {
                    emit_json_or_text(&rows, json, &format!("已发布 {} 条成绩", rows.len()))
                }
                Err(err) => emit_error(&err),
            }
        }
        Some(("amend", args)) => {
            let grade_id = str_arg(args, "grade_id");
            let Ok(new_score) = str_arg(args, "score").parse::<Decimal>() else {
                return usage_err("score 必须是合法数字。");
            };
            let version = parse_u64(args, "version");
            let req = AmendGradeRequest {
                new_score,
                reason: str_arg(args, "reason").to_owned(),
                version,
                idempotency_key: format!("cli-grd-amd-{grade_id}-{version}"),
            };
            match gw.amend_grade(session, grade_id, &req) {
                Ok(updated) => emit_json_or_text(&updated, json, &updated.score.to_string()),
                Err(err) => emit_error(&err),
            }
        }
        Some(("amendments", args)) => {
            let grade_id = args.get_one::<String>("grade_id").map(String::as_str);
            match gw.list_grade_amendments(session, grade_id) {
                Ok(rows) => emit_json_or_text(&rows, json, &format!("共 {} 条修订", rows.len())),
                Err(err) => emit_error(&err),
            }
        }
        _ => 2,
    }
}

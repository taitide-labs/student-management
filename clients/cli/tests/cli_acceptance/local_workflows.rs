use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::Value;

struct TempDb {
    path: PathBuf,
}

fn remove_silent(path: &Path) {
    if fs::remove_file(path).is_err() {}
}

impl TempDb {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("sms_cli_{name}_{}.db", std::process::id()));
        remove_silent(&path);
        Self { path }
    }

    fn arg(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        remove_silent(&self.path);
        remove_silent(&self.path.with_extension("db-wal"));
        remove_silent(&self.path.with_extension("db-shm"));
    }
}

fn sms() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sms"))
}

fn run_json(db: &str, args: &[&str]) -> Value {
    let output = sms()
        .arg("--local")
        .arg("--db")
        .arg(db)
        .arg("--json")
        .args(args)
        .output()
        .expect("run sms command");
    assert!(
        output.status.success(),
        "cmd {args:?} failed stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn str_field(val: &Value, key: &str) -> String {
    val.get(key)
        .and_then(Value::as_str)
        .expect("string field")
        .to_owned()
}

fn setup_teaching_and_student(db: &str) -> (String, String) {
    let hr = run_json(
        db,
        &[
            "homeroom",
            "create",
            "--name",
            "高一 3 班",
            "--grade-year",
            "2026",
            "--capacity",
            "35",
        ],
    );
    assert_eq!(hr.get("name"), Some(&Value::from("高一 3 班")));
    let hr_id = str_field(&hr, "id");

    let stu = run_json(
        db,
        &["student", "enroll", "--name", "张三", "--homeroom", &hr_id],
    );
    let student_no = str_field(&stu, "student_no");

    let crs = run_json(
        db,
        &[
            "course",
            "create",
            "--code",
            "ENG101",
            "--name",
            "大学英语",
            "--max-score",
            "100",
            "--pass-score",
            "60",
        ],
    );
    assert_eq!(crs.get("code"), Some(&Value::from("ENG101")));
    let course_id = str_field(&crs, "id");

    let acc = run_json(
        db,
        &[
            "account",
            "provision",
            "--username",
            "cli_teacher",
            "--display-name",
            "李老师",
            "--role",
            "teacher",
        ],
    );
    assert_eq!(acc.get("username"), Some(&Value::from("cli_teacher")));
    let teacher_id = str_field(&acc, "account_id");

    let tch = run_json(
        db,
        &[
            "teaching",
            "assign",
            "--course",
            &course_id,
            "--homeroom",
            &hr_id,
            "--term",
            "2026-秋",
            "--teacher",
            &teacher_id,
        ],
    );
    (student_no, str_field(&tch, "id"))
}

#[test]
fn help_lists_orthogonal_resources() {
    let output = sms().arg("--help").output().expect("run sms --help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("auth"));
    for cmd in [
        "auth", "account", "homeroom", "student", "course", "teaching", "grade",
    ] {
        assert!(text.contains(cmd), "missing command in help: {cmd}");
    }
}

#[test]
fn homeroom_course_grade_and_account_subcommands_work_end_to_end() {
    let db = TempDb::new("e2e");
    let db_arg = db.arg();
    let (student_no, teaching_id) = setup_teaching_and_student(&db_arg);

    let grd_json = run_json(
        &db_arg,
        &[
            "grade",
            "save",
            "--teaching",
            &teaching_id,
            "--student-no",
            &student_no,
            "--score",
            "91.5",
        ],
    );
    assert_eq!(grd_json.get("score"), Some(&Value::from("91.5")));
}

#[test]
fn student_list_json_outputs_seeded_roster() {
    let db = TempDb::new("list");
    let db_arg = db.arg();

    let hr = run_json(
        &db_arg,
        &[
            "homeroom",
            "create",
            "--name",
            "高一 1 班",
            "--grade-year",
            "2026",
        ],
    );
    let hr_id = str_field(&hr, "id");
    let enrolled = run_json(
        &db_arg,
        &[
            "student",
            "enroll",
            "--name",
            "名单生",
            "--homeroom",
            &hr_id,
        ],
    );
    let stu_no = str_field(&enrolled, "student_no");

    let parsed = run_json(&db_arg, &["student", "list"]);
    let rows = parsed.as_array().expect("student list must be JSON array");
    assert!(
        rows.iter()
            .any(|row| row.get("student_no").and_then(Value::as_str) == Some(stu_no.as_str()))
    );
}

#[test]
fn student_enroll_full_homeroom_exits_four_on_stderr() {
    let db = TempDb::new("cap");
    let db_arg = db.arg();

    let hr = run_json(
        &db_arg,
        &[
            "homeroom",
            "create",
            "--name",
            "容量班",
            "--grade-year",
            "2026",
            "--capacity",
            "1",
        ],
    );
    let hr_id = str_field(&hr, "id");
    let _first = run_json(
        &db_arg,
        &[
            "student",
            "enroll",
            "--name",
            "占满生",
            "--homeroom",
            &hr_id,
        ],
    );

    let output = sms()
        .args([
            "--local",
            "--db",
            &db_arg,
            "student",
            "enroll",
            "--name",
            "满员拦截",
            "--homeroom",
            &hr_id,
        ])
        .output()
        .expect("run sms student enroll");
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stdout.is_empty());
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("homeroom_capacity_exceeded"), "stderr: {err}");
}

#[test]
fn non_tty_withdraw_without_yes_exits_two() {
    let output = sms()
        .args([
            "--local",
            "student",
            "withdraw",
            "20261001",
            "--reason",
            "转学",
            "--version",
            "1",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run sms student withdraw");
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(err.contains("--yes"), "stderr: {err}");
}

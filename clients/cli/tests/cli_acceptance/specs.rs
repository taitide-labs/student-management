use std::process::{Command, Stdio};

use serde_json::Value;

fn sms() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sms"))
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
    let hr_out = sms()
        .args([
            "--local",
            "--json",
            "homeroom",
            "create",
            "--name",
            "高一 3 班",
            "--grade-year",
            "2026",
            "--capacity",
            "35",
        ])
        .output()
        .expect("homeroom create");
    assert!(
        hr_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&hr_out.stderr)
    );
    let hr_json: Value = serde_json::from_slice(&hr_out.stdout).expect("homeroom json");
    assert_eq!(hr_json.get("name"), Some(&Value::from("高一 3 班")));

    let crs_out = sms()
        .args([
            "--local",
            "--json",
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
        ])
        .output()
        .expect("course create");
    assert!(
        crs_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&crs_out.stderr)
    );
    let crs_json: Value = serde_json::from_slice(&crs_out.stdout).expect("course json");
    assert_eq!(crs_json.get("code"), Some(&Value::from("ENG101")));

    let acc_out = sms()
        .args([
            "--local",
            "--json",
            "account",
            "provision",
            "--username",
            "cli_teacher",
            "--display-name",
            "李老师",
            "--role",
            "teacher",
        ])
        .output()
        .expect("account provision");
    assert!(
        acc_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&acc_out.stderr)
    );
    let acc_json: Value = serde_json::from_slice(&acc_out.stdout).expect("account json");
    assert_eq!(acc_json.get("username"), Some(&Value::from("cli_teacher")));

    let grd_out = sms()
        .args([
            "--local",
            "--json",
            "grade",
            "save",
            "--teaching",
            "teach-2",
            "--student-no",
            "20261001",
            "--score",
            "91.5",
        ])
        .output()
        .expect("grade save");
    assert!(
        grd_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&grd_out.stderr)
    );
    let grd_json: Value = serde_json::from_slice(&grd_out.stdout).expect("grade json");
    assert_eq!(grd_json.get("score"), Some(&Value::from("91.5")));
}

#[test]
fn student_list_json_outputs_seeded_roster() {
    let output = sms()
        .args(["--local", "student", "list", "--json"])
        .output()
        .expect("run sms student list --json");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: Value = serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    let rows = parsed.as_array().expect("student list must be JSON array");
    assert!(
        rows.iter()
            .any(|row| row.get("student_no") == Some(&Value::from("20261001")))
    );
}

#[test]
fn student_enroll_full_homeroom_exits_four_on_stderr() {
    let output = sms()
        .args([
            "--local",
            "student",
            "enroll",
            "--name",
            "满员拦截",
            "--homeroom",
            "hr-2026-2",
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

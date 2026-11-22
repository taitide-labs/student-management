use rust_decimal::Decimal;
use student_management_gateway::contract::{PublishGradesRequest, SaveDraftGradeRequest};

pub fn build_save_draft(
    teaching_id: &str,
    student_no: &str,
    score_raw: &str,
    idempotency_key: String,
) -> Result<SaveDraftGradeRequest, String> {
    let tid = teaching_id.trim();
    if tid.is_empty() {
        return Err("请选择教学任务。".to_owned());
    }
    let sno = student_no.trim();
    if sno.is_empty() {
        return Err("请输入学生学号。".to_owned());
    }
    let score = score_raw
        .trim()
        .parse::<Decimal>()
        .map_err(|_| "成绩分值必须是合法数字。".to_owned())?;
    Ok(SaveDraftGradeRequest {
        teaching_id: tid.to_owned(),
        student_no: sno.to_owned(),
        score,
        idempotency_key,
    })
}

pub fn build_publish(
    teaching_id: &str,
    idempotency_key: String,
) -> Result<PublishGradesRequest, String> {
    let tid = teaching_id.trim();
    if tid.is_empty() {
        return Err("暂无可发布的教学任务。".to_owned());
    }
    Ok(PublishGradesRequest {
        teaching_id: tid.to_owned(),
        idempotency_key,
    })
}

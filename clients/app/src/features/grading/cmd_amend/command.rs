use rust_decimal::Decimal;
use student_management_gateway::contract::AmendGradeRequest;

pub fn build_request(
    score_raw: &str,
    reason: &str,
    version: u64,
    idempotency_key: String,
) -> Result<AmendGradeRequest, String> {
    let new_score = score_raw
        .trim()
        .parse::<Decimal>()
        .map_err(|_| "修订后分值必须是合法数字。".to_owned())?;
    let trimmed_reason = reason.trim();
    if trimmed_reason.is_empty() {
        return Err("请填写修订原因。".to_owned());
    }
    Ok(AmendGradeRequest {
        new_score,
        reason: trimmed_reason.to_owned(),
        version,
        idempotency_key,
    })
}

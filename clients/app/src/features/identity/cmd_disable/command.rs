use student_management_gateway::contract::DisableAccountRequest;

pub fn build_request(
    reason: &str,
    version: u64,
    idempotency_key: String,
) -> Result<DisableAccountRequest, String> {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return Err("请填写停用原因。".to_owned());
    }
    Ok(DisableAccountRequest {
        reason: trimmed.to_owned(),
        version,
        idempotency_key,
    })
}

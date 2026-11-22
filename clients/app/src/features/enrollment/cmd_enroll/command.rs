use student_management_gateway::contract::EnrollStudentRequest;

pub fn build_request(
    full_name: &str,
    homeroom_id: Option<&str>,
    idempotency_key: String,
) -> Result<EnrollStudentRequest, String> {
    let trimmed_name = full_name.trim();
    if trimmed_name.is_empty() {
        return Err("请输入学生姓名。".to_owned());
    }
    let Some(room) = homeroom_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return Err("请选择目标行政班。".to_owned());
    };
    Ok(EnrollStudentRequest {
        full_name: trimmed_name.to_owned(),
        homeroom_id: room.to_owned(),
        idempotency_key,
    })
}

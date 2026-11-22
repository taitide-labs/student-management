use student_management_gateway::contract::{CreateHomeroomRequest, UpdateHomeroomCapacityRequest};

pub fn build_create(
    name: &str,
    grade_year_raw: &str,
    capacity_raw: &str,
    idempotency_key: String,
) -> Result<CreateHomeroomRequest, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("请输入班级名称。".to_owned());
    }
    let grade_year = grade_year_raw
        .trim()
        .parse::<u16>()
        .map_err(|_| "入学年份必须为有效数字。".to_owned())?;
    let capacity = parse_optional_capacity(capacity_raw)?;
    Ok(CreateHomeroomRequest {
        name: trimmed.to_owned(),
        grade_year,
        capacity,
        idempotency_key,
    })
}

pub fn build_update_capacity(
    capacity_raw: &str,
    version: u64,
    idempotency_key: String,
) -> Result<UpdateHomeroomCapacityRequest, String> {
    Ok(UpdateHomeroomCapacityRequest {
        capacity: parse_optional_capacity(capacity_raw)?,
        version,
        idempotency_key,
    })
}

fn parse_optional_capacity(raw: &str) -> Result<Option<u32>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<u32>()
        .map(Some)
        .map_err(|_| "额定容量必须为正整数。".to_owned())
}

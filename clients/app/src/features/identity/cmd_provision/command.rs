use student_management_gateway::contract::{ProvisionAccountRequest, Role};

pub fn build_request(
    username: &str,
    display_name: &str,
    role: Role,
    student_no_raw: &str,
    idempotency_key: String,
) -> Result<ProvisionAccountRequest, String> {
    let user = username.trim();
    let name = display_name.trim();
    if user.is_empty() || name.is_empty() {
        return Err("登录用户名与显示姓名均不能为空。".to_owned());
    }
    let bound_student_no = Some(student_no_raw.trim())
        .filter(|v| !v.is_empty())
        .map(str::to_owned);
    if role == Role::Student && bound_student_no.is_none() {
        return Err("学生角色账号必须绑定学号。".to_owned());
    }
    Ok(ProvisionAccountRequest {
        username: user.to_owned(),
        display_name: name.to_owned(),
        role,
        bound_student_no,
        idempotency_key,
    })
}

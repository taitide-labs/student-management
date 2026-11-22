use student_management_gateway::contract::{
    EnrollmentStatus, ReverseEnrollmentRequest, TransferHomeroomRequest, TransitionStudentRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Transfer,
    Suspend,
    Resume,
    Withdraw,
    Graduate,
    Reverse,
}

impl ActionKind {
    #[must_use]
    pub const fn button_label(self) -> &'static str {
        match self {
            Self::Transfer => "转班…",
            Self::Suspend => "休学…",
            Self::Resume => "复学…",
            Self::Withdraw => "退学…",
            Self::Graduate => "毕业…",
            Self::Reverse => "冲正…",
        }
    }

    #[must_use]
    pub const fn dialog_title(self) -> &'static str {
        match self {
            Self::Transfer => "行政转班",
            Self::Suspend => "休学登记",
            Self::Resume => "复学登记",
            Self::Withdraw => "退学登记",
            Self::Graduate => "毕业离校",
            Self::Reverse => "终态冲正",
        }
    }
}

#[must_use]
pub fn allowed_actions(status: EnrollmentStatus) -> Vec<ActionKind> {
    match status {
        EnrollmentStatus::Active => vec![
            ActionKind::Transfer,
            ActionKind::Suspend,
            ActionKind::Withdraw,
            ActionKind::Graduate,
        ],
        EnrollmentStatus::Suspended => vec![ActionKind::Resume, ActionKind::Withdraw],
        EnrollmentStatus::Withdrawn | EnrollmentStatus::Graduated => vec![ActionKind::Reverse],
    }
}

pub fn validate_reason(reason: &str) -> Result<String, String> {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return Err("请填写异动原因。".to_owned());
    }
    Ok(trimmed.to_owned())
}

pub fn build_transfer(
    target_homeroom_id: Option<&str>,
    reason: &str,
    version: u64,
    idempotency_key: String,
) -> Result<TransferHomeroomRequest, String> {
    let room_id = target_homeroom_id
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "请选择目标行政班。".to_owned())?;
    Ok(TransferHomeroomRequest {
        target_homeroom_id: room_id.to_owned(),
        reason: validate_reason(reason)?,
        version,
        idempotency_key,
    })
}

pub fn build_transition(
    reason: &str,
    version: u64,
    idempotency_key: String,
) -> Result<TransitionStudentRequest, String> {
    Ok(TransitionStudentRequest {
        reason: validate_reason(reason)?,
        version,
        idempotency_key,
    })
}

pub fn build_reverse(
    target_status: EnrollmentStatus,
    reason: &str,
    version: u64,
    idempotency_key: String,
) -> Result<ReverseEnrollmentRequest, String> {
    if target_status.is_terminal() {
        return Err("终态冲正目标必须为在籍或休学。".to_owned());
    }
    Ok(ReverseEnrollmentRequest {
        target_status,
        reason: validate_reason(reason)?,
        version,
        idempotency_key,
    })
}

use crate::contract::{
    AccountSession, AccountStatus, AccountSummary, BindPersonProfileRequest,
    ChangeAccountRoleRequest, DisableAccountRequest, GatewayError, ProvisionAccountRequest, Role,
};

use super::store::{MockGateway, problem};

impl MockGateway {
    pub(super) fn list_accounts_impl(
        &self,
        session: &AccountSession,
    ) -> Result<Vec<AccountSummary>, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        Ok(self.accounts.clone())
    }

    pub(super) fn provision_account_impl(
        &mut self,
        session: &AccountSession,
        req: &ProvisionAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let username = req.username.trim();
        let display_name = req.display_name.trim();
        let fp = format!(
            "acc-prov:{username}:{display_name}:{:?}:{:?}",
            req.role, req.bound_student_no
        );
        if let Some(cached) = self.check_idempotent::<AccountSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        if username.is_empty() || display_name.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "账号信息不完整",
                "登录用户名与显示姓名均不能为空。",
            ));
        }
        if self.accounts.iter().any(|a| a.username == username) {
            return Err(problem(
                409,
                "username_conflict",
                "用户名已存在",
                "请使用其他登录用户名。",
            ));
        }
        if let Some(student_no) = &req.bound_student_no
            && !self.students.iter().any(|s| s.student_no == *student_no)
        {
            return Err(problem(
                404,
                "student_not_found",
                "绑定学号不存在",
                "请选择有效的学生档案学号。",
            ));
        }
        let created = AccountSummary {
            account_id: format!("acc-{}", self.accounts.len() + 1),
            username: username.to_owned(),
            display_name: display_name.to_owned(),
            role: req.role,
            bound_student_no: req.bound_student_no.clone(),
            status: AccountStatus::Active,
            version: 1,
        };
        self.accounts.push(created.clone());
        self.record_idempotent(&req.idempotency_key, fp, &created)?;
        Ok(created)
    }

    pub(super) fn bind_person_profile_impl(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &BindPersonProfileRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!("acc-bind:{account_id}:{:?}:{}", req.student_no, req.version);
        if let Some(cached) = self.check_idempotent::<AccountSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        if let Some(student_no) = &req.student_no
            && !self.students.iter().any(|s| s.student_no == *student_no)
        {
            return Err(problem(
                404,
                "student_not_found",
                "学号不存在",
                "待绑定的学生档案不存在。",
            ));
        }
        let account = self
            .accounts
            .iter_mut()
            .find(|a| a.account_id == account_id)
            .ok_or_else(|| problem(404, "account_not_found", "账号不存在", "未找到目标账号。"))?;
        if account.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "账号信息已被更新，请刷新后重试。",
            ));
        }
        account.bound_student_no.clone_from(&req.student_no);
        account.version += 1;
        let updated = account.clone();
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }

    pub(super) fn change_account_role_impl(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &ChangeAccountRoleRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!("acc-role:{account_id}:{:?}:{}", req.role, req.version);
        if let Some(cached) = self.check_idempotent::<AccountSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        let account = self
            .accounts
            .iter_mut()
            .find(|a| a.account_id == account_id)
            .ok_or_else(|| problem(404, "account_not_found", "账号不存在", "未找到目标账号。"))?;
        if account.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "账号信息已被更新，请刷新后重试。",
            ));
        }
        if req.role == Role::Student && account.bound_student_no.is_none() {
            return Err(problem(
                422,
                "validation_failed",
                "学生角色需绑定学号",
                "切换为学生角色前请先关联学生档案。",
            ));
        }
        account.role = req.role;
        account.version += 1;
        let updated = account.clone();
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }

    pub(super) fn disable_account_impl(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &DisableAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let reason = req.reason.trim();
        let fp = format!("acc-dis:{account_id}:{reason}:{}", req.version);
        if let Some(cached) = self.check_idempotent::<AccountSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        if reason.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "停用原因不能为空",
                "停用登录账号必须填写操作原因。",
            ));
        }
        if session.account_id == account_id {
            return Err(problem(
                409,
                "cannot_disable_self",
                "禁止停用当前登录账号",
                "当前操作账号处于登录会话中，不允许自我停用。",
            ));
        }
        let account = self
            .accounts
            .iter_mut()
            .find(|a| a.account_id == account_id)
            .ok_or_else(|| problem(404, "account_not_found", "账号不存在", "未找到目标账号。"))?;
        if account.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "账号信息已被更新，请刷新后重试。",
            ));
        }
        account.status = AccountStatus::Disabled;
        account.version += 1;
        let updated = account.clone();
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }
}

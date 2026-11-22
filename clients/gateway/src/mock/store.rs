use std::collections::HashMap;

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    contract::{
        AccountSession, AccountStatus, AccountSummary, AmendGradeRequest, AssignTeachingRequest,
        BindPersonProfileRequest, ChangeAccountRoleRequest, CourseSummary, CreateCourseRequest,
        CreateHomeroomRequest, DisableAccountRequest, EnrollStudentRequest, EnrollmentChangeRecord,
        EnrollmentStatus, Gateway, GatewayError, GradeAmendmentRecord, GradeEntrySummary,
        GradeStatus, HomeroomSummary, ProblemDetails, ProvisionAccountRequest,
        PublishGradesRequest, ReverseEnrollmentRequest, Role, SaveDraftGradeRequest, SignInRequest,
        StudentDetailPayload, StudentSummary, TeachingSummary, TransferHomeroomRequest,
        TransitionStudentRequest, UpdateHomeroomCapacityRequest,
    },
    mock::seed::{
        seed_accounts, seed_changes, seed_courses, seed_grades, seed_homerooms, seed_students,
        seed_teachings, to_session,
    },
};

#[derive(Debug, Clone)]
pub struct MockGateway {
    pub(super) accounts: Vec<AccountSummary>,
    pub(super) homerooms: Vec<HomeroomSummary>,
    pub(super) students: Vec<StudentSummary>,
    pub(super) changes: Vec<EnrollmentChangeRecord>,
    pub(super) courses: Vec<CourseSummary>,
    pub(super) teachings: Vec<TeachingSummary>,
    pub(super) grades: Vec<GradeEntrySummary>,
    pub(super) grade_amendments: Vec<GradeAmendmentRecord>,
    idempotency: HashMap<String, (String, Value)>,
    next_student_seq: u32,
    simulate_offline: bool,
}

impl Default for MockGateway {
    fn default() -> Self {
        Self::seeded()
    }
}

impl MockGateway {
    #[must_use]
    pub fn seeded() -> Self {
        Self {
            accounts: seed_accounts(),
            homerooms: seed_homerooms(),
            students: seed_students(),
            changes: seed_changes(),
            courses: seed_courses(),
            teachings: seed_teachings(),
            grades: seed_grades(),
            grade_amendments: Vec::new(),
            idempotency: HashMap::new(),
            next_student_seq: 1007,
            simulate_offline: false,
        }
    }

    pub(super) fn ensure_online(&self) -> Result<(), GatewayError> {
        if self.simulate_offline {
            return Err(GatewayError::Transport(
                "连接服务端超时，请检查网络后重试。".to_owned(),
            ));
        }
        Ok(())
    }

    pub(super) fn check_idempotent<T: DeserializeOwned>(
        &self,
        key: &str,
        fingerprint: &str,
    ) -> Result<Option<T>, GatewayError> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "缺少幂等键",
                "写请求必须携带非空幂等键。",
            ));
        }
        if let Some((stored_fp, cached)) = self.idempotency.get(trimmed) {
            if stored_fp == fingerprint {
                let parsed = serde_json::from_value::<T>(cached.clone()).map_err(|err| {
                    GatewayError::Transport(format!("幂等缓存反序列化失败：{err}"))
                })?;
                return Ok(Some(parsed));
            }
            return Err(problem(
                409,
                "idempotency_key_conflict",
                "幂等键冲突",
                "同一幂等键不能用于不同参数荷载的写请求。",
            ));
        }
        Ok(None)
    }

    pub(super) fn record_idempotent<T: Serialize>(
        &mut self,
        key: &str,
        fingerprint: String,
        result: &T,
    ) -> Result<(), GatewayError> {
        let value = serde_json::to_value(result)
            .map_err(|err| GatewayError::Transport(format!("幂等缓存序列化失败：{err}")))?;
        self.idempotency
            .insert(key.trim().to_owned(), (fingerprint, value));
        Ok(())
    }

    pub(super) fn ensure_admin(session: &AccountSession) -> Result<(), GatewayError> {
        if session.role != Role::Admin {
            return Err(problem(
                403,
                "forbidden",
                "权限不足",
                "当前操作仅限教务管理员执行。",
            ));
        }
        Ok(())
    }

    fn find_student_mut(&mut self, student_no: &str) -> Result<usize, GatewayError> {
        self.students
            .iter()
            .position(|s| s.student_no == student_no)
            .ok_or_else(|| problem(404, "student_not_found", "学籍不存在", "未找到对应学号。"))
    }

    fn transition_status(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
        target: EnrollmentStatus,
        action_name: &str,
    ) -> Result<StudentSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!("transition:{student_no}:{target:?}:{}", req.reason.trim());
        if let Some(cached) = self.check_idempotent::<StudentSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        let reason = req.reason.trim();
        if reason.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "异动原因不能为空",
                "学籍状态流转必须附带明确原因。",
            ));
        }
        let idx = self.find_student_mut(student_no)?;
        let current =
            self.students.get(idx).cloned().ok_or_else(|| {
                problem(404, "student_not_found", "学籍不存在", "未找到对应学号。")
            })?;
        if current.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "学籍记录已被其他操作更新，请刷新后重试。",
            ));
        }
        let allowed = matches!(
            (current.status, target),
            (
                EnrollmentStatus::Active,
                EnrollmentStatus::Suspended
                    | EnrollmentStatus::Withdrawn
                    | EnrollmentStatus::Graduated
            ) | (
                EnrollmentStatus::Suspended,
                EnrollmentStatus::Active | EnrollmentStatus::Withdrawn
            )
        );
        if !allowed {
            return Err(problem(
                409,
                "invalid_status_transition",
                "非法学籍状态流转",
                "当前学籍状态不允许直接变更为目标状态。",
            ));
        }
        if target.is_terminal()
            && let Some(room) = self
                .homerooms
                .iter_mut()
                .find(|r| r.id == current.homeroom_id)
        {
            room.active_count = room.active_count.saturating_sub(1);
            room.version += 1;
        }
        let Some(slot) = self.students.get_mut(idx) else {
            return Err(problem(
                404,
                "student_not_found",
                "学籍不存在",
                "未找到对应学号。",
            ));
        };
        let from_summary = format!("{} · {}", slot.homeroom_name, slot.status.label());
        slot.status = target;
        slot.version += 1;
        let updated = slot.clone();
        let to_summary = format!("{} · {}", updated.homeroom_name, updated.status.label());
        self.changes.insert(
            0,
            EnrollmentChangeRecord {
                id: format!("chg-{}", self.changes.len() + 1),
                student_no: updated.student_no.clone(),
                action: action_name.to_owned(),
                from_summary,
                to_summary,
                reason: reason.to_owned(),
                operator_name: session.display_name.clone(),
                occurred_at: "2026-11-22 17:00".to_owned(),
            },
        );
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }
}

impl Gateway for MockGateway {
    fn sign_in(&self, req: &SignInRequest) -> Result<AccountSession, GatewayError> {
        self.ensure_online()?;
        let username = req.username.trim();
        if username.is_empty() || req.password.trim().is_empty() {
            return Err(problem(
                400,
                "invalid_credentials",
                "登录参数缺失",
                "请输入账号与密码。",
            ));
        }
        let account = self
            .accounts
            .iter()
            .find(|acc| acc.username == username && req.password == "demo")
            .ok_or_else(|| {
                problem(
                    401,
                    "unauthorized",
                    "账号或密码错误",
                    "可使用演示账号 admin / teacher / student，密码均为 demo。",
                )
            })?;
        if account.status == AccountStatus::Disabled {
            return Err(problem(
                403,
                "account_disabled",
                "账号已停用",
                "该登录账号已被管理员停用，请联系教务处。",
            ));
        }
        Ok(to_session(account))
    }

    fn switch_role(&self, role: Role) -> Result<AccountSession, GatewayError> {
        self.ensure_online()?;
        self.accounts
            .iter()
            .find(|acc| acc.role == role && acc.status == AccountStatus::Active)
            .map(to_session)
            .ok_or_else(|| {
                problem(
                    404,
                    "account_not_found",
                    "未找到角色账号",
                    "当前桩数据未配置该角色。",
                )
            })
    }

    fn list_homerooms(&self) -> Result<Vec<HomeroomSummary>, GatewayError> {
        self.ensure_online()?;
        Ok(self.homerooms.clone())
    }

    fn create_homeroom(
        &mut self,
        session: &AccountSession,
        req: &CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        self.create_homeroom_impl(session, req)
    }

    fn update_homeroom_capacity(
        &mut self,
        session: &AccountSession,
        homeroom_id: &str,
        req: &UpdateHomeroomCapacityRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        self.update_homeroom_capacity_impl(session, homeroom_id, req)
    }

    fn list_students(
        &self,
        session: &AccountSession,
        query: &str,
        status_filter: Option<EnrollmentStatus>,
    ) -> Result<Vec<StudentSummary>, GatewayError> {
        self.ensure_online()?;
        let keyword = query.trim().to_lowercase();
        let rows = self
            .students
            .iter()
            .filter(|student| {
                if session.role == Role::Student {
                    return session
                        .bound_student_no
                        .as_deref()
                        .is_some_and(|no| no == student.student_no);
                }
                true
            })
            .filter(|student| status_filter.is_none_or(|st| student.status == st))
            .filter(|student| {
                keyword.is_empty()
                    || student.student_no.to_lowercase().contains(&keyword)
                    || student.full_name.to_lowercase().contains(&keyword)
                    || student.homeroom_name.to_lowercase().contains(&keyword)
            })
            .cloned()
            .collect();
        Ok(rows)
    }

    fn student_detail(
        &self,
        session: &AccountSession,
        student_no: &str,
    ) -> Result<StudentDetailPayload, GatewayError> {
        self.ensure_online()?;
        if session.role == Role::Student && session.bound_student_no.as_deref() != Some(student_no)
        {
            return Err(problem(
                403,
                "forbidden",
                "无权查看他人学籍",
                "学生账号仅可查看本人学籍档案。",
            ));
        }
        let student = self
            .students
            .iter()
            .find(|s| s.student_no == student_no)
            .cloned()
            .ok_or_else(|| problem(404, "student_not_found", "学籍不存在", "未找到对应学号。"))?;
        let changes = self
            .changes
            .iter()
            .filter(|c| c.student_no == student_no)
            .cloned()
            .collect();
        Ok(StudentDetailPayload { student, changes })
    }

    fn enroll_student(
        &mut self,
        session: &AccountSession,
        req: &EnrollStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!("enroll:{}:{}", req.full_name.trim(), req.homeroom_id);
        if let Some(cached) = self.check_idempotent::<StudentSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        let full_name = req.full_name.trim();
        if full_name.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "姓名不能为空",
                "请填写学生真实姓名。",
            ));
        }
        let homeroom = self
            .homerooms
            .iter_mut()
            .find(|room| room.id == req.homeroom_id)
            .ok_or_else(|| {
                problem(
                    404,
                    "homeroom_not_found",
                    "行政班不存在",
                    "请选择有效的目标行政班。",
                )
            })?;
        if let Some(cap) = homeroom.capacity
            && homeroom.active_count >= cap
        {
            return Err(problem(
                409,
                "homeroom_capacity_exceeded",
                "行政班已满员",
                "目标行政班人数已达额定上限，请选择其他班级或调大班额。",
            ));
        }

        let student_no = format!("2026{:04}", self.next_student_seq);
        self.next_student_seq += 1;
        homeroom.active_count += 1;
        homeroom.version += 1;

        let student = StudentSummary {
            student_no: student_no.clone(),
            full_name: full_name.to_owned(),
            homeroom_id: homeroom.id.clone(),
            homeroom_name: homeroom.name.clone(),
            status: EnrollmentStatus::Active,
            enrolled_on: "2026-11-22".to_owned(),
            version: 1,
        };
        self.students.insert(0, student.clone());
        self.changes.insert(
            0,
            EnrollmentChangeRecord {
                id: format!("chg-{}", self.changes.len() + 1),
                student_no,
                action: "入学建档".to_owned(),
                from_summary: "无".to_owned(),
                to_summary: format!("{} · 在籍", homeroom.name),
                reason: "新生统一建档".to_owned(),
                operator_name: session.display_name.clone(),
                occurred_at: "2026-11-22 15:50".to_owned(),
            },
        );
        self.record_idempotent(&req.idempotency_key, fp, &student)?;
        Ok(student)
    }

    fn transfer_homeroom(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransferHomeroomRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!("transfer:{student_no}:{}", req.target_homeroom_id);
        if let Some(cached) = self.check_idempotent::<StudentSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        let idx = self.find_student_mut(student_no)?;
        let current =
            self.students.get(idx).cloned().ok_or_else(|| {
                problem(404, "student_not_found", "学籍不存在", "未找到对应学号。")
            })?;
        if current.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "学籍记录已变更，请刷新后重试。",
            ));
        }
        if current.status != EnrollmentStatus::Active {
            return Err(problem(
                409,
                "invalid_status_transition",
                "仅在籍学生可转班",
                "非在籍状态不支持直接转班。",
            ));
        }
        let target_idx = self
            .homerooms
            .iter()
            .position(|r| r.id == req.target_homeroom_id)
            .ok_or_else(|| {
                problem(
                    404,
                    "homeroom_not_found",
                    "目标行政班不存在",
                    "请选择有效行政班。",
                )
            })?;
        let target_room =
            self.homerooms.get(target_idx).cloned().ok_or_else(|| {
                problem(404, "homeroom_not_found", "行政班不存在", "未找到行政班。")
            })?;
        if let Some(cap) = target_room.capacity
            && target_room.active_count >= cap
        {
            return Err(problem(
                409,
                "homeroom_capacity_exceeded",
                "目标行政班已满员",
                "目标行政班人数已达额定上限。",
            ));
        }
        if let Some(old_room) = self
            .homerooms
            .iter_mut()
            .find(|r| r.id == current.homeroom_id)
        {
            old_room.active_count = old_room.active_count.saturating_sub(1);
            old_room.version += 1;
        }
        if let Some(new_room) = self.homerooms.get_mut(target_idx) {
            new_room.active_count += 1;
            new_room.version += 1;
        }
        let Some(slot) = self.students.get_mut(idx) else {
            return Err(problem(
                404,
                "student_not_found",
                "学籍不存在",
                "未找到对应学号。",
            ));
        };
        let from_summary = format!("{} · {}", slot.homeroom_name, slot.status.label());
        slot.homeroom_id = target_room.id;
        slot.homeroom_name = target_room.name;
        slot.version += 1;
        let updated = slot.clone();
        self.changes.insert(
            0,
            EnrollmentChangeRecord {
                id: format!("chg-{}", self.changes.len() + 1),
                student_no: updated.student_no.clone(),
                action: "行政转班".to_owned(),
                from_summary,
                to_summary: format!("{} · {}", updated.homeroom_name, updated.status.label()),
                reason: req.reason.trim().to_owned(),
                operator_name: session.display_name.clone(),
                occurred_at: "2026-11-22 17:00".to_owned(),
            },
        );
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }

    fn suspend_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.transition_status(
            session,
            student_no,
            req,
            EnrollmentStatus::Suspended,
            "休学登记",
        )
    }

    fn resume_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.transition_status(
            session,
            student_no,
            req,
            EnrollmentStatus::Active,
            "复学登记",
        )
    }

    fn withdraw_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.transition_status(
            session,
            student_no,
            req,
            EnrollmentStatus::Withdrawn,
            "退学登记",
        )
    }

    fn graduate_student(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &TransitionStudentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.transition_status(
            session,
            student_no,
            req,
            EnrollmentStatus::Graduated,
            "毕业离校",
        )
    }

    fn reverse_enrollment(
        &mut self,
        session: &AccountSession,
        student_no: &str,
        req: &ReverseEnrollmentRequest,
    ) -> Result<StudentSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!(
            "reverse:{student_no}:{:?}:{}",
            req.target_status,
            req.reason.trim()
        );
        if let Some(cached) = self.check_idempotent::<StudentSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        let reason = req.reason.trim();
        if reason.is_empty() || req.target_status.is_terminal() {
            return Err(problem(
                422,
                "validation_failed",
                "冲正参数无效",
                "终态冲正必须写明原因并指定非终态目标。",
            ));
        }
        let idx = self.find_student_mut(student_no)?;
        let current =
            self.students.get(idx).cloned().ok_or_else(|| {
                problem(404, "student_not_found", "学籍不存在", "未找到对应学号。")
            })?;
        if current.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "学籍记录已变更，请刷新后重试。",
            ));
        }
        if !current.status.is_terminal() {
            return Err(problem(
                409,
                "invalid_status_transition",
                "非终态无需冲正",
                "仅退学或毕业终态支持管理员冲正。",
            ));
        }
        if let Some(room) = self
            .homerooms
            .iter_mut()
            .find(|r| r.id == current.homeroom_id)
        {
            if let Some(cap) = room.capacity
                && room.active_count >= cap
            {
                return Err(problem(
                    409,
                    "homeroom_capacity_exceeded",
                    "原行政班已满员",
                    "原行政班已满额，无法直接冲正回籍。",
                ));
            }
            room.active_count += 1;
            room.version += 1;
        }
        let Some(slot) = self.students.get_mut(idx) else {
            return Err(problem(
                404,
                "student_not_found",
                "学籍不存在",
                "未找到对应学号。",
            ));
        };
        let from_summary = format!("{} · {}", slot.homeroom_name, slot.status.label());
        slot.status = req.target_status;
        slot.version += 1;
        let updated = slot.clone();
        self.changes.insert(
            0,
            EnrollmentChangeRecord {
                id: format!("chg-{}", self.changes.len() + 1),
                student_no: updated.student_no.clone(),
                action: "终态冲正".to_owned(),
                from_summary,
                to_summary: format!("{} · {}", updated.homeroom_name, updated.status.label()),
                reason: reason.to_owned(),
                operator_name: session.display_name.clone(),
                occurred_at: "2026-11-22 17:00".to_owned(),
            },
        );
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }

    fn list_courses(&self, session: &AccountSession) -> Result<Vec<CourseSummary>, GatewayError> {
        self.list_courses_impl(session)
    }

    fn create_course(
        &mut self,
        session: &AccountSession,
        req: &CreateCourseRequest,
    ) -> Result<CourseSummary, GatewayError> {
        self.create_course_impl(session, req)
    }

    fn list_teachings(
        &self,
        session: &AccountSession,
    ) -> Result<Vec<TeachingSummary>, GatewayError> {
        self.list_teachings_impl(session)
    }

    fn assign_teaching(
        &mut self,
        session: &AccountSession,
        req: &AssignTeachingRequest,
    ) -> Result<TeachingSummary, GatewayError> {
        self.assign_teaching_impl(session, req)
    }

    fn list_grades(
        &self,
        session: &AccountSession,
        student_no: Option<&str>,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        self.ensure_online()?;
        let rows =
            self.grades
                .iter()
                .filter(|g| match session.role {
                    Role::Student => {
                        session
                            .bound_student_no
                            .as_deref()
                            .is_some_and(|no| no == g.student_no)
                            && g.status == GradeStatus::Published
                    }
                    Role::Teacher => self.teachings.iter().any(|t| {
                        t.id == g.teaching_id && t.teacher_account_id == session.account_id
                    }),
                    Role::Admin => true,
                })
                .filter(|g| student_no.is_none_or(|no| g.student_no == no))
                .cloned()
                .collect();
        Ok(rows)
    }

    fn save_draft_grade(
        &mut self,
        session: &AccountSession,
        req: &SaveDraftGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        self.save_draft_grade_impl(session, req)
    }

    fn publish_grades(
        &mut self,
        session: &AccountSession,
        req: &PublishGradesRequest,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        self.publish_grades_impl(session, req)
    }

    fn amend_grade(
        &mut self,
        session: &AccountSession,
        grade_id: &str,
        req: &AmendGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        self.amend_grade_impl(session, grade_id, req)
    }

    fn list_grade_amendments(
        &self,
        session: &AccountSession,
        grade_id: Option<&str>,
    ) -> Result<Vec<GradeAmendmentRecord>, GatewayError> {
        self.list_grade_amendments_impl(session, grade_id)
    }

    fn list_accounts(&self, session: &AccountSession) -> Result<Vec<AccountSummary>, GatewayError> {
        self.list_accounts_impl(session)
    }

    fn provision_account(
        &mut self,
        session: &AccountSession,
        req: &ProvisionAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.provision_account_impl(session, req)
    }

    fn bind_person_profile(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &BindPersonProfileRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.bind_person_profile_impl(session, account_id, req)
    }

    fn change_account_role(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &ChangeAccountRoleRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.change_account_role_impl(session, account_id, req)
    }

    fn disable_account(
        &mut self,
        session: &AccountSession,
        account_id: &str,
        req: &DisableAccountRequest,
    ) -> Result<AccountSummary, GatewayError> {
        self.disable_account_impl(session, account_id, req)
    }

    fn set_offline(&mut self, offline: bool) {
        self.simulate_offline = offline;
    }
}

pub(super) fn problem(status: u16, code: &str, title: &str, detail: &str) -> GatewayError {
    GatewayError::Problem(ProblemDetails {
        problem_type: format!("https://student-management.local/errors/{code}"),
        title: title.to_owned(),
        status,
        detail: detail.to_owned(),
        code: code.to_owned(),
    })
}

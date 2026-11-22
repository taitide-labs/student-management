use rust_decimal::Decimal;

use crate::contract::{
    AccountSession, AmendGradeRequest, AssignTeachingRequest, CourseSummary, CreateCourseRequest,
    EnrollmentStatus, GatewayError, GradeAmendmentRecord, GradeEntrySummary, GradeStatus,
    PublishGradesRequest, Role, SaveDraftGradeRequest, TeachingSummary,
};

use super::store::{MockGateway, problem};

impl MockGateway {
    pub(super) fn list_courses_impl(
        &self,
        _session: &AccountSession,
    ) -> Result<Vec<CourseSummary>, GatewayError> {
        self.ensure_online()?;
        Ok(self.courses.clone())
    }

    pub(super) fn create_course_impl(
        &mut self,
        session: &AccountSession,
        req: &CreateCourseRequest,
    ) -> Result<CourseSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let code = req.code.trim().to_uppercase();
        let name = req.name.trim();
        let fp = format!(
            "crs-create:{code}:{name}:{}:{}",
            req.max_score, req.pass_score
        );
        if let Some(cached) = self.check_idempotent::<CourseSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        if code.is_empty()
            || name.is_empty()
            || req.max_score <= Decimal::ZERO
            || req.pass_score < Decimal::ZERO
            || req.pass_score > req.max_score
        {
            return Err(problem(
                422,
                "validation_failed",
                "课程规则无效",
                "课程编码、名称必填且及格线不能超过满分。",
            ));
        }
        if self.courses.iter().any(|c| c.code == code) {
            return Err(problem(
                409,
                "course_code_conflict",
                "课程编码重复",
                "相同课程编码已存在。",
            ));
        }
        let created = CourseSummary {
            id: format!("crs-{}", code.to_lowercase()),
            code,
            name: name.to_owned(),
            max_score: req.max_score,
            pass_score: req.pass_score,
            version: 1,
        };
        self.courses.push(created.clone());
        self.record_idempotent(&req.idempotency_key, fp, &created)?;
        Ok(created)
    }

    pub(super) fn list_teachings_impl(
        &self,
        session: &AccountSession,
    ) -> Result<Vec<TeachingSummary>, GatewayError> {
        self.ensure_online()?;
        let rows = match session.role {
            Role::Admin => self.teachings.clone(),
            Role::Teacher => self
                .teachings
                .iter()
                .filter(|t| t.teacher_account_id == session.account_id)
                .cloned()
                .collect(),
            Role::Student => Vec::new(),
        };
        Ok(rows)
    }

    pub(super) fn assign_teaching_impl(
        &mut self,
        session: &AccountSession,
        req: &AssignTeachingRequest,
    ) -> Result<TeachingSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let term = req.term.trim();
        let fp = format!(
            "tch-assign:{}:{}:{term}:{}",
            req.course_id, req.homeroom_id, req.teacher_account_id
        );
        if let Some(cached) = self.check_idempotent::<TeachingSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        if term.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "学期不能为空",
                "请指定开课学期。",
            ));
        }
        let course = self
            .courses
            .iter()
            .find(|c| c.id == req.course_id)
            .cloned()
            .ok_or_else(|| problem(404, "course_not_found", "课程不存在", "未找到指定课程。"))?;
        let homeroom = self
            .homerooms
            .iter()
            .find(|r| r.id == req.homeroom_id)
            .cloned()
            .ok_or_else(|| problem(404, "homeroom_not_found", "行政班不存在", "未找到行政班。"))?;
        let teacher = self
            .accounts
            .iter()
            .find(|a| a.account_id == req.teacher_account_id && a.role == Role::Teacher)
            .cloned()
            .ok_or_else(|| {
                problem(
                    422,
                    "invalid_teacher",
                    "任课教师无效",
                    "请选择有效教师账号。",
                )
            })?;
        let assigned = TeachingSummary {
            id: format!("teach-{}", self.teachings.len() + 1),
            course_id: course.id,
            course_code: course.code,
            course_name: course.name,
            homeroom_id: homeroom.id,
            homeroom_name: homeroom.name,
            term: term.to_owned(),
            teacher_account_id: teacher.account_id,
            teacher_name: teacher.display_name,
            version: 1,
        };
        self.teachings.push(assigned.clone());
        self.record_idempotent(&req.idempotency_key, fp, &assigned)?;
        Ok(assigned)
    }

    pub(super) fn save_draft_grade_impl(
        &mut self,
        session: &AccountSession,
        req: &SaveDraftGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        self.ensure_online()?;
        let fp = format!(
            "grd-save:{}:{}:{}",
            req.teaching_id, req.student_no, req.score
        );
        if let Some(cached) =
            self.check_idempotent::<GradeEntrySummary>(&req.idempotency_key, &fp)?
        {
            return Ok(cached);
        }
        let teaching = self.authorized_teaching(session, &req.teaching_id)?;
        let course = self
            .courses
            .iter()
            .find(|c| c.id == teaching.course_id)
            .cloned()
            .ok_or_else(|| {
                problem(
                    404,
                    "course_not_found",
                    "课程不存在",
                    "教学任务关联课程缺失。",
                )
            })?;
        if req.score < Decimal::ZERO || req.score > course.max_score {
            return Err(problem(
                422,
                "score_out_of_range",
                "分值超出课程范围",
                "成绩分值须介于 0 与课程满分之间。",
            ));
        }
        let student = self
            .students
            .iter()
            .find(|s| s.student_no == req.student_no)
            .cloned()
            .ok_or_else(|| problem(404, "student_not_found", "学籍不存在", "未找到对应学生。"))?;
        if student.status != EnrollmentStatus::Active {
            return Err(problem(
                409,
                "student_not_active",
                "非在籍学生禁止录分",
                "仅在籍状态学生允许录入课程成绩。",
            ));
        }
        let saved = if let Some(existing) = self
            .grades
            .iter_mut()
            .find(|g| g.teaching_id == teaching.id && g.student_no == student.student_no)
        {
            if existing.status == GradeStatus::Published {
                return Err(problem(
                    409,
                    "grade_already_published",
                    "成绩已锁定发布",
                    "已发布成绩需由教务管理员发起合规修订。",
                ));
            }
            existing.score = req.score;
            existing.status = GradeStatus::Draft;
            existing.version += 1;
            existing.clone()
        } else {
            let entry = GradeEntrySummary {
                id: format!("grd-{}", self.grades.len() + 1),
                teaching_id: teaching.id,
                course_code: teaching.course_code,
                course_name: teaching.course_name,
                term: teaching.term,
                student_no: student.student_no,
                student_name: student.full_name,
                score: req.score,
                max_score: course.max_score,
                status: GradeStatus::Draft,
                version: 1,
            };
            self.grades.push(entry.clone());
            entry
        };
        self.record_idempotent(&req.idempotency_key, fp, &saved)?;
        Ok(saved)
    }

    pub(super) fn publish_grades_impl(
        &mut self,
        session: &AccountSession,
        req: &PublishGradesRequest,
    ) -> Result<Vec<GradeEntrySummary>, GatewayError> {
        self.ensure_online()?;
        let fp = format!("grd-pub:{}", req.teaching_id);
        if let Some(cached) =
            self.check_idempotent::<Vec<GradeEntrySummary>>(&req.idempotency_key, &fp)?
        {
            return Ok(cached);
        }
        let _teaching = self.authorized_teaching(session, &req.teaching_id)?;
        let mut updated = Vec::new();
        for grade in &mut self.grades {
            if grade.teaching_id == req.teaching_id {
                if grade.status == GradeStatus::Draft {
                    grade.status = GradeStatus::Published;
                    grade.version += 1;
                }
                updated.push(grade.clone());
            }
        }
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }

    pub(super) fn amend_grade_impl(
        &mut self,
        session: &AccountSession,
        grade_id: &str,
        req: &AmendGradeRequest,
    ) -> Result<GradeEntrySummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let reason = req.reason.trim();
        let fp = format!("grd-amd:{grade_id}:{}:{reason}", req.new_score);
        if let Some(cached) =
            self.check_idempotent::<GradeEntrySummary>(&req.idempotency_key, &fp)?
        {
            return Ok(cached);
        }
        if reason.is_empty() {
            return Err(problem(
                422,
                "validation_failed",
                "修订原因不能为空",
                "锁分后合规修订必须写明具体原因。",
            ));
        }
        let grade = self
            .grades
            .iter_mut()
            .find(|g| g.id == grade_id)
            .ok_or_else(|| problem(404, "grade_not_found", "成绩条目不存在", "未找到成绩记录。"))?;
        if grade.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "成绩记录已被其他操作更新，请刷新后重试。",
            ));
        }
        if grade.status != GradeStatus::Published {
            return Err(problem(
                409,
                "grade_not_published",
                "仅已锁定成绩可修订",
                "草稿阶段请直接修改并保存成绩。",
            ));
        }
        if req.new_score < Decimal::ZERO || req.new_score > grade.max_score {
            return Err(problem(
                422,
                "score_out_of_range",
                "分值超出课程范围",
                "修订后分值须介于 0 与课程满分之间。",
            ));
        }
        let prev = grade.score;
        grade.score = req.new_score;
        grade.version += 1;
        let updated = grade.clone();
        self.grade_amendments.insert(
            0,
            GradeAmendmentRecord {
                id: format!("amd-{}", self.grade_amendments.len() + 1),
                grade_id: updated.id.clone(),
                course_name: updated.course_name.clone(),
                student_no: updated.student_no.clone(),
                student_name: updated.student_name.clone(),
                previous_score: prev,
                new_score: updated.score,
                reason: reason.to_owned(),
                operator_name: session.display_name.clone(),
                occurred_at: "2026-11-22 17:30".to_owned(),
            },
        );
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }

    pub(super) fn list_grade_amendments_impl(
        &self,
        session: &AccountSession,
        grade_id: Option<&str>,
    ) -> Result<Vec<GradeAmendmentRecord>, GatewayError> {
        self.ensure_online()?;
        if session.role == Role::Student {
            return Err(problem(
                403,
                "forbidden",
                "权限不足",
                "学生不可查看成绩合规修订台账。",
            ));
        }
        Ok(self
            .grade_amendments
            .iter()
            .filter(|a| grade_id.is_none_or(|id| a.grade_id == id))
            .cloned()
            .collect())
    }

    fn authorized_teaching(
        &self,
        session: &AccountSession,
        teaching_id: &str,
    ) -> Result<TeachingSummary, GatewayError> {
        if session.role == Role::Student {
            return Err(problem(
                403,
                "forbidden",
                "权限不足",
                "学生账号禁止维护课程成绩。",
            ));
        }
        let teaching = self
            .teachings
            .iter()
            .find(|t| t.id == teaching_id)
            .cloned()
            .ok_or_else(|| {
                problem(
                    404,
                    "teaching_not_found",
                    "教学任务不存在",
                    "未找到授课分派。",
                )
            })?;
        if session.role == Role::Teacher && teaching.teacher_account_id != session.account_id {
            return Err(problem(
                403,
                "forbidden",
                "无权操作他人教学任务",
                "教师仅可录入并发布本人分派的课程成绩。",
            ));
        }
        Ok(teaching)
    }
}

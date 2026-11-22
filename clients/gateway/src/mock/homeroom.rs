use crate::contract::{
    AccountSession, CreateHomeroomRequest, GatewayError, HomeroomSummary,
    UpdateHomeroomCapacityRequest,
};

use super::store::{MockState, problem};

impl MockState {
    pub(super) fn create_homeroom_impl(
        &mut self,
        session: &AccountSession,
        req: &CreateHomeroomRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let name = req.name.trim();
        let fp = format!("hr-create:{name}:{}:{:?}", req.grade_year, req.capacity);
        if let Some(cached) = self.check_idempotent::<HomeroomSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        if name.is_empty() || req.grade_year == 0 {
            return Err(problem(
                422,
                "validation_failed",
                "行政班参数无效",
                "班级名称与入学年份不能为空。",
            ));
        }
        if self.homerooms.iter().any(|r| r.name == name) {
            return Err(problem(
                409,
                "homeroom_name_conflict",
                "行政班已存在",
                "同名行政班已存在。",
            ));
        }
        let room = HomeroomSummary {
            id: format!("hr-{}-{}", req.grade_year, self.homerooms.len() + 1),
            name: name.to_owned(),
            grade_year: req.grade_year,
            capacity: req.capacity,
            active_count: 0,
            version: 1,
        };
        self.homerooms.push(room.clone());
        self.record_idempotent(&req.idempotency_key, fp, &room)?;
        Ok(room)
    }

    pub(super) fn update_homeroom_capacity_impl(
        &mut self,
        session: &AccountSession,
        homeroom_id: &str,
        req: &UpdateHomeroomCapacityRequest,
    ) -> Result<HomeroomSummary, GatewayError> {
        self.ensure_online()?;
        Self::ensure_admin(session)?;
        let fp = format!("hr-cap:{homeroom_id}:{:?}:{}", req.capacity, req.version);
        if let Some(cached) = self.check_idempotent::<HomeroomSummary>(&req.idempotency_key, &fp)? {
            return Ok(cached);
        }
        let room = self
            .homerooms
            .iter_mut()
            .find(|r| r.id == homeroom_id)
            .ok_or_else(|| {
                problem(
                    404,
                    "homeroom_not_found",
                    "行政班不存在",
                    "未找到对应班级。",
                )
            })?;
        if room.version != req.version {
            return Err(problem(
                409,
                "version_conflict",
                "数据版本冲突",
                "行政班信息已被其他操作更新，请刷新后重试。",
            ));
        }
        if let Some(cap) = req.capacity
            && cap < room.active_count
        {
            return Err(problem(
                409,
                "homeroom_capacity_below_active_count",
                "容量不能低于当前在籍人数",
                format!(
                    "新设定额定容量（{cap}）低于当前在籍人数（{}）。",
                    room.active_count
                )
                .as_str(),
            ));
        }
        room.capacity = req.capacity;
        room.version += 1;
        let updated = room.clone();
        self.record_idempotent(&req.idempotency_key, fp, &updated)?;
        Ok(updated)
    }
}

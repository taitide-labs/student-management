use std::num::NonZeroU32;

use student_management_primitives::{GradeYear, HomeroomName, PositiveVersion};

use crate::error::EnrollmentDomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Homeroom {
    pub id: String,
    pub name: HomeroomName,
    pub grade_year: GradeYear,
    pub capacity: Option<NonZeroU32>,
    pub version: PositiveVersion,
}

impl Homeroom {
    #[must_use]
    pub fn create(
        id: String,
        name: HomeroomName,
        grade_year: GradeYear,
        capacity: Option<NonZeroU32>,
    ) -> Self {
        Self {
            id,
            name,
            grade_year,
            capacity,
            version: PositiveVersion::try_new(1).unwrap_or_else(|_| unreachable!()),
        }
    }

    pub fn ensure_seat_available(&self, active_count: u32) -> Result<(), EnrollmentDomainError> {
        if let Some(cap) = self.capacity
            && active_count >= cap.get()
        {
            return Err(EnrollmentDomainError::HomeroomCapacityExceeded {
                capacity: cap.get(),
                active_count,
            });
        }
        Ok(())
    }

    pub fn update_capacity(
        &mut self,
        expected_version: PositiveVersion,
        new_capacity: Option<NonZeroU32>,
        active_count: u32,
    ) -> Result<(), EnrollmentDomainError> {
        if self.version != expected_version {
            return Err(EnrollmentDomainError::VersionConflict {
                expected: expected_version.get(),
                actual: self.version.get(),
            });
        }
        if let Some(cap) = new_capacity
            && cap.get() < active_count
        {
            return Err(EnrollmentDomainError::CapacityBelowActiveCount {
                capacity: cap.get(),
                active_count,
            });
        }
        self.capacity = new_capacity;
        self.version = self.version.next();
        Ok(())
    }
}

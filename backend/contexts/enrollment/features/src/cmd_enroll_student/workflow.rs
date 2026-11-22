use student_management_contracts::ProblemDetails;
use student_management_enrollment_domain::{EnrollmentChangeDraft, Homeroom, Student};
use student_management_primitives::StudentNo;

use super::command::EnrollStudentCommand;

pub fn enroll(
    homeroom: &Homeroom,
    active_count: u32,
    student_no: StudentNo,
    cmd: &EnrollStudentCommand,
    today_iso: String,
) -> Result<(Student, EnrollmentChangeDraft), ProblemDetails> {
    homeroom
        .ensure_seat_available(active_count)
        .map_err(|err| ProblemDetails::conflict("homeroom_capacity_exceeded", err.to_string()))?;
    Ok(Student::enroll(
        student_no,
        cmd.full_name.clone(),
        homeroom.id.as_str(),
        today_iso,
    ))
}

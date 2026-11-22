use student_management_enrollment_domain::Homeroom;

use super::command::CreateHomeroomCommand;

#[must_use]
pub fn prepare(homeroom_id: String, cmd: &CreateHomeroomCommand) -> Homeroom {
    Homeroom::create(homeroom_id, cmd.name.clone(), cmd.grade_year, cmd.capacity)
}

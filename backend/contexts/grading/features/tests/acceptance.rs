#[cfg(test)]
#[path = "acceptance/support.rs"]
mod support;

#[cfg(test)]
#[path = "acceptance/cmd_create_course.rs"]
mod cmd_create_course;

#[cfg(test)]
#[path = "acceptance/cmd_assign_teaching.rs"]
mod cmd_assign_teaching;

#[cfg(test)]
#[path = "acceptance/cmd_save_draft_grade.rs"]
mod cmd_save_draft_grade;

#[cfg(test)]
#[path = "acceptance/cmd_amend_grade.rs"]
mod cmd_amend_grade;

#[cfg(test)]
#[path = "acceptance/evt_void_draft_grades.rs"]
mod evt_void_draft_grades;

#[cfg(test)]
#[path = "acceptance/support.rs"]
mod support;

#[cfg(test)]
#[path = "acceptance/cmd_homeroom.rs"]
mod cmd_homeroom;

#[cfg(test)]
#[path = "acceptance/cmd_enroll_student.rs"]
mod cmd_enroll_student;

#[cfg(test)]
#[path = "acceptance/cmd_transfer_homeroom.rs"]
mod cmd_transfer_homeroom;

#[cfg(test)]
#[path = "acceptance/cmd_transition_student.rs"]
mod cmd_transition_student;

#[cfg(test)]
#[path = "acceptance/cmd_reverse_enrollment.rs"]
mod cmd_reverse_enrollment;

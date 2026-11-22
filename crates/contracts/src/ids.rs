pub trait IdGenerator {
    fn next_id(&mut self, prefix: &str) -> String;
    fn next_student_no(&mut self, grade_year: u16) -> String;
}

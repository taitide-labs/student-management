CREATE TABLE IF NOT EXISTS homerooms (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    grade_year INTEGER NOT NULL CHECK (grade_year BETWEEN 1900 AND 2999),
    capacity INTEGER CHECK (capacity IS NULL OR capacity >= 1),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (name, grade_year)
);

CREATE TABLE IF NOT EXISTS student_no_sequences (
    grade_year INTEGER PRIMARY KEY NOT NULL,
    last_seq INTEGER NOT NULL CHECK (last_seq >= 0)
);

CREATE TABLE IF NOT EXISTS students (
    student_no TEXT PRIMARY KEY NOT NULL,
    full_name TEXT NOT NULL,
    homeroom_id TEXT NOT NULL REFERENCES homerooms(id),
    status TEXT NOT NULL CHECK (status IN ('active', 'suspended', 'withdrawn', 'graduated')),
    enrolled_on TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_students_homeroom_status ON students(homeroom_id, status);

CREATE TABLE IF NOT EXISTS enrollment_change_records (
    id TEXT PRIMARY KEY NOT NULL,
    student_no TEXT NOT NULL REFERENCES students(student_no),
    action TEXT NOT NULL,
    from_summary TEXT NOT NULL,
    to_summary TEXT NOT NULL,
    reason TEXT NOT NULL,
    operator_name TEXT NOT NULL,
    occurred_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_enrollment_changes_student ON enrollment_change_records(student_no, occurred_at);

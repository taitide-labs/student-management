CREATE TABLE IF NOT EXISTS courses (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    max_score TEXT NOT NULL,
    pass_score TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS teaching_assignments (
    id TEXT PRIMARY KEY NOT NULL,
    course_id TEXT NOT NULL REFERENCES courses(id),
    homeroom_id TEXT NOT NULL,
    term TEXT NOT NULL,
    teacher_account_id TEXT NOT NULL,
    teacher_name TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    UNIQUE (course_id, homeroom_id, term)
);

CREATE INDEX IF NOT EXISTS idx_teaching_teacher ON teaching_assignments(teacher_account_id);
CREATE INDEX IF NOT EXISTS idx_teaching_homeroom ON teaching_assignments(homeroom_id);

CREATE TABLE IF NOT EXISTS grade_entries (
    id TEXT PRIMARY KEY NOT NULL,
    teaching_id TEXT NOT NULL REFERENCES teaching_assignments(id),
    student_no TEXT NOT NULL,
    score TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('draft', 'published', 'voided')),
    version INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL,
    UNIQUE (teaching_id, student_no)
);

CREATE INDEX IF NOT EXISTS idx_grade_entries_student ON grade_entries(student_no);

CREATE TABLE IF NOT EXISTS grade_amendments (
    id TEXT PRIMARY KEY NOT NULL,
    grade_id TEXT NOT NULL REFERENCES grade_entries(id),
    previous_score TEXT NOT NULL,
    new_score TEXT NOT NULL,
    reason TEXT NOT NULL,
    operator_name TEXT NOT NULL,
    occurred_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_grade_amendments_grade ON grade_amendments(grade_id, occurred_at);

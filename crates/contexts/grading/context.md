# 课程与成绩

核心域，负责课程设置、教学任务分派，以及成绩录入、发布与合规修订。

## 术语

**课程**：带有统一编码、名称及满分与及格阈值规则的标准化教学科目。

- 等价：`Course`
- 避免：课节、课时、课表

**教学任务**：特定学期内由指定教师面向特定行政班标识开设的课程授课实例。

- 等价：`TeachingAssignment`
- 避免：行政班、排课表、教学班

**成绩单条目**：特定学生在特定教学任务下的考核分数及其发布状态的评估记录。

- 等价：`GradeEntry`
- 避免：原始分、总评、成绩单

**成绩修订记录**：成绩锁定发布后由合规授权发起的、携带原值、新值与修订原因的替代性更正凭证。

- 等价：`GradeAmendment`
- 避免：改分日志、覆盖记录

## Workflow

- `CreateCourse`：`CreateCourseCommand` -> `CourseCreated`
- `AssignTeaching`：`AssignTeachingCommand` -> `TeachingAssigned`
- `SaveDraftGrade`：`SaveDraftGradeCommand` -> `DraftGradeSaved`
- `PublishGrades`：`PublishGradesCommand` -> `GradesPublished`
- `AmendGrade`：`AmendGradeCommand` -> `GradeAmended`
- `VoidDraftGradesOnWithdrawal`：`StudentWithdrawn`（订阅集成事件）-> `DraftGradesVoided`

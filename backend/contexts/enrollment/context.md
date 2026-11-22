# 学籍与行政班

核心域，负责行政班管理、学生入学建档与分班，以及学籍变更与异动追溯。

## 术语

**学生**：受学籍生命周期约束的受教育个体档案。

- 等价：`Student`
- 避免：用户、学员、账号

**学号**：学生在校期间及离校后用于唯一标识其档案的编号。

- 等价：`StudentNo`
- 避免：编号、登录名、账号标识

**行政班**：具有额定容量约束的固定学生编制集体，同一在籍学生仅归属一个行政班。

- 等价：`Homeroom`
- 避免：教学班、课程班、分组

**学籍状态**：表征学生当前在校资格与允许流转范围的生命周期标识。

- 等价：`EnrollmentStatus`
- 避免：账号状态、启用状态

**学籍异动记录**：承载入学、转班、休学、复学、退学、毕业及误操作冲正业务事实的凭证。

- 等价：`EnrollmentChangeRecord`
- 避免：操作日志、修改记录

## Workflow

- `CreateHomeroom`：`CreateHomeroomCommand` -> `HomeroomCreated`
- `UpdateHomeroomCapacity`：`UpdateHomeroomCapacityCommand` -> `HomeroomCapacityUpdated`
- `EnrollStudent`：`EnrollStudentCommand` -> `StudentEnrolled`
- `TransferHomeroom`：`TransferHomeroomCommand` -> `StudentTransferred`
- `SuspendStudent`：`SuspendStudentCommand` -> `StudentSuspended`
- `ResumeStudent`：`ResumeStudentCommand` -> `StudentResumed`
- `WithdrawStudent`：`WithdrawStudentCommand` -> `StudentWithdrawn`
- `GraduateStudent`：`GraduateStudentCommand` -> `StudentGraduated`
- `ReverseEnrollmentStatus`：`ReverseEnrollmentStatusCommand` -> `EnrollmentStatusReversed`

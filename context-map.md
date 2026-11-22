# 上下文地图

## 限界上下文

- [学籍与行政班](./crates/contexts/enrollment/context.md) — 核心域，负责行政班管理、学生入学建档与分班，以及学籍变更与异动追溯
- [课程与成绩](./crates/contexts/grading/context.md) — 核心域，负责课程设置、教学任务分派，以及成绩录入、发布与合规修订
- [身份与权限](./crates/contexts/identity/context.md) — 通用域，负责系统账号、访问凭证、预置角色以及账号与人员身份的关联管理

## 关系模式

- 学籍与行政班 [OHS, PL] -> 课程与成绩 [ACL]
  - 事件：发布 `StudentWithdrawn`，下游订阅触发 `VoidDraftGradesOnWithdrawal`
  - 同步端口：调用 `EnrollmentQueryPort` 完成学生是否在籍、所属行政班及班级存在性的即时校验
- 身份与权限 [OHS, PL] -> 课程与成绩 [ACL]
  - 同步端口：调用 `IdentityLookupPort` 完成教学任务指派中任课教师账号有效性与教师角色校验

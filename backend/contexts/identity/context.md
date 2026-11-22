# 身份与权限

通用域，负责系统账号、访问凭证、预置角色以及账号与人员身份的关联管理。

## 术语

**账号**：带有登录名、启用状态与归属角色的独立认证主体实体，与业务学籍档案解耦。

- 等价：`Account`
- 避免：学生、用户档案、学号

**角色**：界定系统访问许可类别的预置权限标识。

- 等价：`Role`
- 避免：用户组、权限点、职务

**人员档案绑定**：账号与校内学生业务标识之间的显式关联关系。

- 等价：`PersonProfileBinding`
- 避免：学籍档案、主数据合并

## Workflow

- `ProvisionAccount`：`ProvisionAccountCommand` -> `AccountProvisioned`
- `BindPersonProfile`：`BindPersonProfileCommand` -> `PersonProfileBound`
- `ChangeAccountRole`：`ChangeAccountRoleCommand` -> `AccountRoleChanged`
- `DisableAccount`：`DisableAccountCommand` -> `AccountDisabled`

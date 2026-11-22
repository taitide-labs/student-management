# 任务拆解与工单编排

遵循 Inverse Conway Maneuver 与 SSOT 原则，基于已合入主干的冻结基线，通过临时工单拓扑 YAML 与本地脚本在 GitHub 上批量创建层级工单及阻塞依赖图，以隔离并发实现代理的读写作用域。

工单正文只描述业务目标、规则边界与代码库路径坐标，将工作树操作、格式化与门禁命令留给 `implement` Skill 与自动化流水线，以保持工单高信噪比。

## 执行流程

1. 核对前置基线：确认本次交付涉及的冻结基线（数据库迁移脚本、共享内核、切片契约入口与挂起 `wip` 跳过标记的黑盒验收测试）已合入主干。
2. 编排并落盘临时工单拓扑 YAML：
   - 将工单拓扑写入临时文件（如 `/tmp/<delivery-slug>-issues.yaml`）；严禁将该临时文件提交入库，必须仅作为创建工单前审阅与脚本输入使用。
   - 当交付仅涉及单个独立后端或命令行切片且无需端到端联调时，仅定义 1 条垂直切片工单（按需填写 `milestone`），不创建 Parent Issue 与末尾整体验收工单。
   - 当涉及多个切片协同或需要前后端联动联调时，按需填写目标版本的 `milestone`，定义 1 条 Parent Issue 概括整体业务闭环，再挂载各并行切片 Sub-issue（通过 `parent` 关联）与 1 条末尾整体验收 Sub-issue：
     - 末尾整体验收工单在 `blocked_by` 中列出全部并行切片 `id`，形成有向无环图。
3. 预审校验并批量创建工单：
   - 将临时 YAML 文件路径与工单依赖摘要呈交给用户审阅；根据反馈修改 YAML 直至用户确认。
   - 严禁阅读脚本源码，必须直接调用 `design` Skill 目录下的 `./scripts/create-issues.ts`（可用 `--help` 查看用法），仅当需要修改脚本逻辑时读取。
   - 先执行 `./scripts/create-issues.ts --dry-run /tmp/<delivery-slug>-issues.yaml` 校验字段完整与依赖无环；确认无误后去掉 `--dry-run` 批量创建 Milestone、Parent Issue、Sub-issue 与阻塞依赖，完成后删除临时 YAML 文件。

## YAML 结构与正文模板

临时工单拓扑文件 `/tmp/<delivery-slug>-issues.yaml`：

```yaml
milestone: <目标版本号，如 v0.1.0；可选>
issues:
  - id: <parent-id>
    title: "<需求交付名称>"
    body: |
      <1-3 句话概括本次交付覆盖的业务链路与整体完成定义>

      ## 上下文与契约

      - 领域上下文：<context-map.md 与相关 context.md 路径>
      - 契约与基线：<contracts/、DESIGN.md 或数据库迁移目录路径>

  - id: <slice-id>
    parent: <parent-id；单切片交付时省略>
    blocked_by: [<前置依赖 id 或既有工单号；无前置依赖时省略>]
    title: "<业务动作与领域对象名称>"
    body: |
      <1-3 句话描述业务规则与异常拒止边界>

      ## 指向坐标

      - 接口与切片契约：<contracts/ 或切片入参出参契约路径>
      - 共享基线：<数据库迁移脚本或共享 domain 路径；不涉及时省略>
      - 切片目录：<后端切片或命令行子命令所在目标目录>

      ## 验收场景

      - 待激活：<验收测试路径 :: scenario_fn_1>
      - 待激活：<验收测试路径 :: scenario_fn_2>
      - 授权移除：<验收测试路径 :: superseded_scenario_fn> <!-- 仅当存量行为变更取代旧规则时填写，否则省略本行 -->

  - id: <e2e-id>
    parent: <parent-id>
    blocked_by: [<全部并行切片 id>]
    title: "<需求交付名称> 端到端整体验收"
    body: |
      <1-2 句话说明激活跨上下文集成旅程测试、导出契约并派生客户端类型校验兼容性、切换真实服务连接及激活前端入口或清理旧实现（如有）的联调目标>

      ## 验收与联调坐标

      - 端到端旅程测试：<端到端旅程测试目录或文件路径；不涉及时省略>
      - 契约与客户端：<contracts/openapi.json、客户端协议与连接配置路径、待激活入口或待清理旧实现路径；不涉及时省略>
```

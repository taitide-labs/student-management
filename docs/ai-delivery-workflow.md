# AI 协同交付与全链路测试时机工作流

本文档沉淀多 Agent（架构 AI / 切片 AI / 集成 AI）协同研发的阶段边界、并发安全区间，以及各类测试的**责任归属、编写时机与冻结稳定性判定**。

---

## 1. 五阶段协同流水线与并行隔离边界

```mermaid
flowchart TD
    S1["阶段 1：战略设计\n划分上下文与统一语言\n产出：context-map.md / context.md"] --> S2["阶段 2：客户端原型与契约冻结\nMock UI 交互定稿 + 冻结契约\n产出：contracts / migrations / openapi / Gateway"]
    S2 --> S3["阶段 3：战术基线与黑盒验收冻结\n共享 Domain 单测 + Planned 路由 + wip 切片验收测试\n产出：domain / infra / tests/acceptance/*.rs"]
    S3 --> S4["阶段 4：切片并行实现\n一切片一 AI / 一工作树\n只实现本切片代码 + 按需单测 + 取消 wip 标记"]
    S4 --> S5["阶段 5：跨上下文整合与跨端 E2E 收尾\n激活 runtime 跨上下文测试 + 页面入口跨端 E2E"]
```

### 并发边界与冲突热点控制

| 阶段 | 负责角色 | 允许改动范围 | 并行安全度 | 说明 |
| :--- | :--- | :--- | :--- | :--- |
| **阶段 1–3** | 架构/主干 AI | `context*.md`、`migrations/`、`backend/{primitives,contracts,infra,api}`、`domain/`、`clients/{gateway,app}` 基础契约与 `tests/acceptance/*.rs` | **严格串行（主干冻结）** | 共享契约未冻结前严禁派发并行切片 AI，否则下游大面积返工。 |
| **阶段 4** | 垂直切片 AI | 单个 `<ctx>/features/src/<slice>/`（及对应 `endpoint.rs`、`acceptance/<slice>.rs`） | **100% 并行零冲突** | 阶段 3 冻结 `handler::execute` 空签名存根与 `gateway` 转发后，切片 AI 只填实自己目录内的实现，不再修改 `runtime` 或 `gateway`。 |
| **阶段 5** | 集成收尾 AI | `backend/runtime/tests/`、`clients/{cli,app}` 真实模式配置、`e2e/tests/` | **单点汇聚** | 全部切片合入后再接入真实客户端配置与跨端 E2E 闭环。 |

---

## 2. 全栈测试编写时机与“前置稳定性”判定矩阵

评估一项测试**能不能在切片代码尚未实现之前就稳定写出来**，第一性原理只有一个标准：**它依赖的契约（DTO、数据库表、UI 无障碍标签、或内部函数签名）是否已经冻结**。

| 测试类型 | 物理目录 | 谁来写 | 何时写 | 切片实现前能否写出稳定测试？ | 依赖前提与原因 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. 原语/共享领域单元测试** | `backend/primitives/src/tests/`<br>`backend/contexts/*/domain/src/tests/` | 架构/内核 AI | **阶段 3**（随共享内核冻结即写即绿） | **直接写且立即可跑通** | 纯函数与不变量先于切片冻结，是切片的基石。 |
| **2. 后端切片单元测试** | `contexts/*/features/src/<slice>/tests/` | **切片 AI** | **阶段 4**（按需编写） | **不能，严禁提前写** | 切片内部的 `command.rs` / `workflow.rs` / helper 签名属于切片内部重构自由度；只测值得测的纯规则计算。 |
| **3. 客户端切片单元测试** | `clients/app/src/features/<ctx>/<slice>/tests.rs` | 客户端设计 AI | **阶段 2**（随前端原型同步写出） | **直接写且立即可跑通** | 前端输入校验与命令组装只依赖 `contracts` DTO，不依赖后端是否实现。 |
| **4. 后端切片黑盒验收测试** | `contexts/*/features/tests/acceptance/<slice>.rs` | 架构/基线 AI | **阶段 3 写出（标 `#[ignore = "wip"]`），阶段 4 激活** | **能！非常稳定** | 仅通过 HTTP `oneshot` 测输入/输出 JSON、状态码、幂等与 DB 结果；跨上下文 Port 用 `Fake*Port` 隔绝，完全不碰切片内部代码。 |
| **5. API / Gateway 契约测试** | `backend/api/src/tests/`<br>`clients/gateway/src/tests/` | 架构/客户端 AI | **阶段 2–3** | **能（按 `Active` 数量渐进调整）** | 守卫 `OpenAPI` 结构、`Planned` 隐藏策略与 `MockGateway` 契约一致。 |
| **6. Page UI 验收测试** | `clients/app/src/pages/<page>/*.test.tsx` | 客户端设计 AI | **阶段 2**（随原型即写即绿） | **能！非常稳定** | 通过 `mise run client`（`nsh test clients/app`）无头运行，后端由 `MockGateway` 承接，独立于后端切片实现。 |
| **7. CLI 本地子进程验收** | `clients/cli/tests/cli_acceptance/` | CLI/集成 AI | **阶段 2 写命令框架测试，阶段 5 接真实 DB** | **仅参数/退出码部分稳定；真 DB 需待切片激活** | 早期依赖 `MockGateway`；切到 `--local --db` 真实运行时后，必须依赖后端切片与合法主键/幂等键才能稳定通过。 |
| **8. `runtime` 跨上下文测试** | `backend/runtime/tests/` | 架构/集成 AI | **阶段 3 可先留 `#[ignore = "wip"]` 场景契约，阶段 5 激活** | **行为可预写，断言依赖全部关联切片激活** | 直接通过 `AppState` 调用跨上下文 `handler::execute` 与 `dispatch_outbox`（不经 HTTP），需相关上下文全部切片激活后转绿。 |
| **9. 页面入口跨端 E2E 测试** | `e2e/tests/` | 集成 AI | **阶段 5**（切片与远端网关全部就绪后） | **不建议过早写死细节，应在阶段 5 落地** | 驱动真实 Page (`nsh_testing::Headless`) 经 `NshRemoteGateway`（`UreqNetwork`）访问真实 API Server 与 SQLite；可通过 `mise run e2e` 单独复验。 |

---

## 3. 工程执行规范

1. **保持 `runtime` 为纯组合根**
   - 在阶段 3 冻结全部切片的 `handler::execute` 签名、`endpoint.rs`（`Planned`）以及 `clients/gateway` 转发；阶段 4 多工作树并行期间，切片实现仅触及自身切片目录与对应 `acceptance` 测试。
   - 本地部署形态通过 `LocalGatewayHandle` 直接调度 `slice::handler::execute`，不建立中心透传包装层。

2. **客户端网络接入与 Mock 边界**
   - 页面、功能切片与命令仅依赖 `Gateway` 异步特征和 `contracts` DTO，严禁硬编码 Mock 主键、默认演示密码或仅 Mock 可用的快捷入口。
   - `clients/app` 通过 `nsh_core::prelude::global` 管理当前引擎实例的网关句柄，E2E 与多实例宿主经 `EngineContext` 注入 `RemoteEndpoint`，禁止在业务或页面层引入 `thread_local!` 或跨实例全局可变静态变量。

3. **幂等键与测试池事务纪律**
   - 客户端构造幂等键统一进行空白转义与长度截断，符合 `IdempotencyKey` 校验规范。
   - 需要 `#[rstest]` 参数化矩阵的切片验收测试统一采用 `#[tokio::test]` 与 `open_sqlite_pool("sqlite::memory:")`；所有外部 `Port` 查询必须在数据库事务开始前完成，以避免 `:memory:` 单连接池死锁。

4. **分级校验命令**
   - 日常快速校验：`mise run check` 执行文档、`ast-grep`、Clippy、工作区 Rust 测试与依赖安全审计。
   - 客户端界面验收：修改 `clients/app` 或 `clients/gateway` 后，按需执行 `mise run client`（`nsh test clients/app`）。
   - 跨端 E2E 复验：调整真实 HTTP 链路或页面流程后，执行 `mise run e2e`。

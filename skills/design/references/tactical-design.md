# 战术设计与验收冻结

遵循 Modular Monolith、VSA、DMMF、CQRS 融合架构与 Trunk-Based Development，在主干先行冻结后端或命令行的共享内核、切片契约入口与黑盒验收测试基线，以消除多工作树并行合并冲突并保留切片内部重构自由。

严禁提前编写切片内部的 `workflow` 业务编排、`handler` 实现及切片白盒单元测试，必须仅冻结共享内核、切片契约入口与挂起 `wip` 跳过标记的黑盒验收测试。

## 执行流程

按本次变更波及范围依序冻结下列基线（存量变更未涉及的层级保持不动；若在阶段 3 闭环后于新会话切入，直接读取 `main` 上的 `context.md`、`context-map.md` 与已验收的客户端协议层对齐接口事实）：

1. 落盘底层骨架：创建全局递增数据库迁移脚本（按需）、后端或命令行工作区模块、发版清单、统一门禁入口（如 `mise.toml` 的 `fmt` 与 `check` 任务，需确保隔离 `Worktree` 缺少未追踪依赖目录时可自动补齐或隔离就绪）与静态防线基线（含跨上下文隔离、同上下文跨切片互引禁令、读切片禁引写侧领域模型、禁止内联测试及禁止普通代码注释规则）。
2. 实现共享内核与测试：完成并测通 `primitives`、`contracts`、基础设施层及各上下文共享 `domain` 的测试；当增量改造既有共享模型或表结构时，遵循 Parallel Change 按扩展、迁移、收缩保留兼容层。
3. 冻结 Keystone Interface：
   - 为新增垂直切片冻结入参出参契约及初始标记为未激活（`Planned`）的自注册路由或命令入口，确保生产构建屏蔽未完工入口且多切片并行期间零中央路由合并冲突。
   - 当修改存量切片时，严禁将既有入口退回 `Planned` 下线线上功能，必须保持入口为 `Active` 并按 Parallel Change 扩展契约。
   - 当冻结中暴露出领域歧义或接口缺口时，回退至战略设计或契约设计修订。
4. 冻结黑盒验收测试：
   - 依据 Specification by Example 与 Given-When-Then，基于隔离临时环境与参数化边界矩阵，为各切片编写覆盖正常流与异常拒止流的黑盒验收测试。
   - 当涉及命令行工具时，验收测试同步覆盖子命令路由、退出码及 `stdout` 与 `stderr` 流隔离；当涉及多上下文交付时，同步冻结跨上下文端到端旅程测试骨架。
   - 既有通过断言保持不动，新增未实现用例统一挂起 `wip` 跳过标记，以防下游 Reward Hacking；当存量行为变更将取代旧业务规则的既有断言时，保持旧断言在主干继续通过并新增 `wip` 新规则断言，记录被取代的旧场景名交由阶段 5 工单显式授权下游移除。
5. 通过 PR 合入主干：严禁直接推送 `main`，必须在独立分支通过 `mise run fmt` 与 `mise run check` 全量门禁后创建 PR 合入 `main`。

## 代码放置与架构细则

- 跨界原语与协议分层：当值对象、强类型标识或交互协议跨越多个限界上下文时，归入全局 `primitives`（仅存放私有字段配校验构造入口的受限业务原语，避免退化为通用工具集合）或 `contracts`（仅存放同步 `Port`、开放 `IntegrationEvent` 与非确定性基础设施抽象）。
- 上下文共享领域核：当纯状态转化与核心不变量在同一限界上下文内被两个及以上写切片共享时，归入该上下文的共享 `domain` 层，遵循 Making Illegal States Unrepresentable 与 Persistence Ignorance 保持零外部 I/O 依赖。
- 写切片隔离：当代码服务于写用例时，归入以 `cmd_` 为前缀的独立垂直切片目录，严禁同上下文跨切片互引；遵循 Parse, Don't Validate 在入口将原始输入一次性解析为受限命令，按 Functional Core, Imperative Shell 将复杂纯计算提炼为无副作用 `workflow`，由命令式 `handler` 编排外部端口与本地事务。
- 读切片隔离：当代码服务于查询用例时，归入以 `qry_` 为前缀的独立垂直切片目录；严禁导入写侧 `domain` 模型或跨切片互引，必须按 CQRS 直接执行只读视图投影返回响应，以避免冗余的领域对象组装。
- 枚举开闭纪律：跨越上下文与切片边界分发的生命周期状态和集成事件采用开放枚举，强制下游匹配保留未知变体兜底分支；上下文内部的领域失败原因与同步 `Port` 错误采用封闭枚举，强制每个切片入口显式穷尽映射全部失败分支。
- 注释与自解释代码：严禁编写普通代码注释，应当通过命名与受限类型表达意图；局部反直觉行为或外部缺陷绕过应当通过命名精准的回归测试锁定，非显而易见的架构决策写入 ADR；仅允许编译器或工具链消费的结构化注释（如 `///` 契约文档、`// SAFETY:` 与带 `reason` 的编译豁免）。

## 物理拓扑与编译约束

以 Rust 技术栈示例，其他编程语言等义迁移，按下列物理拓扑与工具链约束落盘：

```text
workspace-root/
├── context-map.md
├── mise.toml                         # 统一格式化与全量静态及测试门禁入口
├── docs/adr/                         # 全局架构决策记录
├── contracts/openapi.json            # 聚合契约导出产物（按需）
├── migrations/                       # 全局递增 SQL 迁移脚本（按需）
├── rules/                            # ast-grep 架构边界与禁注释静态规则
└── crates/
    ├── primitives/                   # 受限基础值原语，私有字段与 try_new
    ├── contracts/                    # 跨上下文同步端口与 #[non_exhaustive] 集成事件
    ├── infra/                        # AppState、EventBus、SliceDescriptor 注册表
    ├── contexts/<ctx>/
    │   ├── context.md
    │   ├── domain/                   # 共享纯状态机与封闭错误枚举
    │   │   └── src/tests/<unit>.rs   # 共享领域单元测试
    │   └── slices/
    │       ├── tests/
    │       │   ├── acceptance.rs     # 单测试根入口，仅声明 #[path] 子模块
    │       │   └── acceptance/<slice>.rs
    │       └── src/
    │           ├── lib.rs            # 纯切片声明与 link() 链接锚点
    │           ├── <slice>.rs        # 纯子模块声明
    │           └── <slice>/
    │               ├── tests.rs      # 单元测试声明容器（按需）
    │               └── tests/<unit>.rs
    ├── server/                       # 宿主组合根与 OpenAPI 导出（或 cli 入口）
    └── e2e-journeys/                 # 跨上下文端到端旅程测试
```

- 包粒度控制：每个限界上下文最多拆分 `domain` 与 `slices` 两个 Crate（纯查询上下文可省略 `domain`），避免每切片独立建包造成运行时重复链接。
- 验收测试聚合：黑盒验收测试统一由单一集成测试目标 `tests/acceptance.rs` 通过 `#[path = "acceptance/<slice>.rs"]` 聚合，使用 `#[sqlx::test]` 与 `#[rstest]` 编写，并在未实现场景标注 `#[ignore = "wip"]`。
- 非叶声明与叶实现分离：严禁使用 `mod.rs` 或在叶子业务源码中内联 `#[test]` 与 `mod tests` 块，必须将 `lib.rs`、`<slice>.rs`、`tests.rs` 与 `tests/acceptance.rs` 限定为纯 `mod` 与 `pub use` 声明容器，并将单元测试独立落盘于 `src/tests/<unit>.rs` 或 `src/<slice>/tests/<unit>.rs`。
- 零冲突路由自注册：各切片在 `endpoint.rs` 中通过 `inventory::submit!` 提交初始状态为 `SliceStatus::Planned` 的路由描述符，并在 `slices/src/lib.rs` 暴露空函数 `link()` 供宿主显式调用一次，防止静态链接阶段消除注册段。

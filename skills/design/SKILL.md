---
name: design
description: 从战略设计至战术设计阶段推进软件设计；当涉及新项目设计、增量需求、变更需求时必须使用，包括后端设计、客户端设计、契约设计，其中客户端实现属于本 Skill，但后端实现不属于。
---

## 核心原则

- 将客户端实现作为设计的一部分，用户需要依赖交互来判断你理解的需求是否与他达成共识，避免返工成本过高。
- 当项目为纯后端时，跳过交互设计阶段；当项目为纯客户端时，推进至阶客户端设计结束；当处理的是增量、变更需求时，跳过示范交付阶段。其他场景一律走完成流程，严禁跨阶段越序执行，必须经用户确认当前阶段完成，而后再进入下一阶段；当执行中需要修订上游事实源时，从受波及的最上游阶段重新切入。
- 当任意阶段存在未决分叉时，发起前沿追问。
- 当任意阶段决策同时满足难逆转、脱离背景令人费解、存在真实方案权衡三项条件时，即刻落盘 ADR；缺任一项时跳过。

> [!TIP]
> 客户端指代交互层而非 OS，即包含 Desktop、Mobile、Web、Mini Program、TUI、CLI。

## 前置流程

若仓库未初始化，必须查看 `` 并完成初始化。

## 阶段流程

1. 战略设计：划分业务子域、限界上下文与统一语言，产出 `context.md`。读取 [strategic-design.md](references/strategic-design.md)。
2. 客户端与契约设计：产出完整前端供用户交互确认并冻结南北或东西向契约。读取 [frontend-and-contract.md](references/client-and-contract.md)
3. 战术设计：冻结基础设施、领域模型、数据库与验收测试。
4. 任务拆解：编排需求 Issue 拓扑，发布至 Github 管理。
5. 示范交付：使用 `implement` Skill 交付首个 Walking Skeleton 需求，合入后结束。

> [!TIP]
> 当处理的是增量、变更需求时，需要查看 `context.md` 或 `context-map.md` 及其阶段所需的上下文，评估变动范围。

## 设计树前沿追问

将所有未决分叉组织为一棵有向无环设计树，每确认一个节点即向下游解锁新的子节点；按轮次推进前沿追问与方案推荐，直至当前讨论范围内的全部关键分支达成共识。

- 当用户提问超出当前阶段边界时，拒绝展开回答，将其拉回当前主线，告知用户将问题延至某某阶段回答，以防过早展开污染当前上下文。
- 严禁向用户询问可通过代码、配置、文档或工具核实的客观事实，必须先查清后再向用户呈现真正需要决策的问题，以避免无效问题或非最佳实践方案干扰用户决策。
- 仅展示前置条件均已确定的前沿问题；当某个问题依赖本轮另一项未决答案，推迟到下一轮确认。
- 用户可能仅挑非推荐方案问题回答，未回答问题视为选择了推荐方案。
- 每轮最多展示 5 个当前前沿问题，每轮编号重置为 `Q1` 至 `Q5`：

  ```markdown
    Q<编号> - <问题标题>
    
    <问题描述>
    
    <方案描述，可使用 Mermaid 呈现；若有多方案，按字母顺序编号为 A/B/C>
    
    推荐：<推荐方案与一句核心理由；单个方案时省去此行>
    
    <多个问题时使用水平分隔线>
  ```

## ADR

- 全局决策存于 `docs/adr/NNNN-slug.md`，限界上下文专属决策存于 `<module_path>/docs/adr/NNNN-slug.md`。
- 编号单调递增，新建状态固定为 `accepted`；当既有决策因业务下线直接作废且无继任方案时改为 `deprecated`；当既有决策被新决策推翻时，新增 ADR 并将旧篇状态改为 `superseded by [NNNN-slug.md](./NNNN-slug.md)`。

```markdown
# <决策简短标题>

Status: <accepted | deprecated | superseded by [NNNN-slug.md](./NNNN-slug.md)>

<1-3 句话简要说明背景、决策与缘由>

## 备选方案

<仅当被否决方案值得保留教训时填写，否则省略本节>

## 影响

<仅当存在非显而易见的架构后果时填写，否则省略本节>
```

## 项目结构大纲

后端以 Rust 为例，客户端以 Rust NSH 为例，其他编程语言或框架等义迁移。

```text
project/
├── mise.toml                         # 统一命令执行网关
├── context-map.md                    # 全局上下文地图
├── Cargo.toml                        # 工作区清单与 lint 开关
├── clippy.toml                       # clippy lint 配置
├── rules/                            # ast-grep lint 配置
├── docs/adr/                         # 全局架构决策记录
├── contracts/                        # 跨端契约导出产物
├── migrations/                       # SQL 迁移脚本
├── e2e/                              # 全栈端对端测试
├── backend/
│   ├── primitives/                   # [Crate] 受限基础值原语
│   ├── contracts/                    # [Crate] 跨上下文通信契约
│   ├── infra/                        # [Crate] 基础设施适配
│   ├── api/                          # [Crate] 对外 API
│   ├── runtime/                      # [Crate: bin] 无协议组合根
│   │   └── tests/                    # 后端端对端测试
│   └── contexts/<ctx>/
│       ├── context.md                # 领域上下文说明
│       ├── docs/adr/                 # 领域架构决策记录
│       ├── domain/                   # [Crate: <ctx>-domain] 纯函数领域模型
│       └── features/                 # [Crate: <ctx>-features] 业务功能层
│           ├── tests/                # 功能验收测试
│           └── src/
│               ├── <cmd_feature>/    # 写功能
│               │   ├── endpoint.rs   # 协议入口
│               │   ├── command.rs    # 命令输入定义
│               │   ├── workflow.rs   # 纯函数业务流
│               │   ├── handler.rs    # 副作用编排
│               │   └── tests/        # 单元测试
│               └── <qry_feature>/    # 读功能
│                   ├── endpoint.rs   # 协议入口
│                   ├── query.rs      # 数据直查投影
│                   └── handler.rs    # 查询参数与视图定义
└── clients/                          # 客户端
    ├── gateway/                      # [Crate] 客户端统一调用网关
    │   ├── remote/                   # 对接 backend/api
    │   ├── local/                    # 对接 backend/runtime
    │   └── mock/                     # mock 数据
    ├── cli/                          # [Crate]
    └── app/                          # [Crate]
        ├── DESIGN.md                 # 视觉规范、语义色使用规则与动效清单
        ├── Cargo.toml
        └── src/
            ├── app/                  # 组合根
            │   ├── init.rs           # 环境初始化
            │   ├── nav.rs            # 强类型 Route 与导航栈
            │   └── shell.rs          # 跨端导航壳
            ├── pages/<feature>_page/  # 按用户屏幕组织，自由拼装多个 features
            │       ├── page.rs        # 
            │       └── tests.rs       # 验收测试
            ├── features/<feature>/
            │       ├── store.rs        # 本组视图共享的响应式缓存
            │       ├── parts/          # 本组内复用的纯展示小块
            │       ├── <cmd_feature>/  # 写交互
            │       │   ├── command.rs  #
            │       │   ├── view.rs     #
            │       │   └── tests.rs    # 单元测试
            │       └── <qry_feature>/  # 读交互
            │           ├── query.rs    # 
            │           └── view.rs     #
            └── ui/                     # 设计系统层
                ├── token.rs            # 语义化颜色、字号、圆角、动效 Tokens
                ├── platform.rs/        # Desktop / Mobile / Web / TUI 交互适配封装
                └── ...                 # button.rs / input.rs / dialog.rs / card.rs 等基础控件
```

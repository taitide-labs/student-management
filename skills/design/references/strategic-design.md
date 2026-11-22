# 战略设计

遵循 DMMF（Domain Modeling Made Functional）、Big Picture Event Storming 与 SSOT 收敛业务边界与领域语义，保持各限界上下文自治。

## 执行流程

1. 切分子域：
   1. 围绕决定竞争壁垒的核心域发起前沿追问。
   2. 将支持域与通用域打包为行业最佳实践方案摘要供用户默认采纳，仅在用户提出异议时展开。
2. 收敛 Hotspots：沿时间轴拉出 `Command -> Workflow -> DomainEvent` 主干链路，针对状态锁定、并发竞争与异步失败补偿识别 Hotspots 并逐轮收敛业务规则。
3. 划分限界上下文并对齐统一语言：
   - 默认从根目录单一 `context.md` 起步；仅当出现一词多义或术语名称成簇时，拆分为 `context-map.md`，拆分时还需分析 ADR 是否跟随拆分。
   - 提炼本上下文特有的业务概念，锁定统一语言，排除超时、重试、分页等通用编程词汇。
   - 按行业标准划分限界上下文，并映射 OHS、PL、ACL、CF。

## 产物模板

根目录 `context.md` 或上下文目录 `<module_path>/context.md`：

```markdown
# <上下文名称>

<1-2 句话简要说明该限界上下文定位及纳入的支撑域、通用域默认摘要>

## 术语

**<术语名称>**：<一句话定义它是什么，严禁写做什么>
- 等价：<CodeIdentifier；当术语本身即为英文时省略本行>
- 避免：<易混词 1>、<易混词 2>

## Workflow

- `<WorkflowName>`：`<Command>` -> `<DomainEvent>`
```

根目录 `context-map.md`：

```markdown
# 上下文地图

## 限界上下文

- [<上下文名称>](./<module_path>/context.md) — <职责与子域分类>

## 关系模式

- <上游上下文> [<上游角色，如 OHS, PL>] -> <下游上下文> [<下游角色，如 ACL, CF>]
  - 事件：发布 `<DomainEvent>`，下游订阅触发 `<WorkflowName>`
  - 同步端口：调用 `<PortTrait>` 完成即时校验、预占或查询
```

> [!TIP]
> 同步端口在战略阶段允许写成协作意图，待契约阶段回填端口名

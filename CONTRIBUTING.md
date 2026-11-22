# 贡献指南

> 本仓库目前仅接受内部维护者的 Pull Request，暂不接受外部 PR。欢迎通过 Issue 反馈 Bug 或功能建议。

本仓库使用 `mise` 管理工具链、使用 `hk` 编排本地 Git Hooks，所有内部变更均通过 Pull Request 合入 `main` 分支。

## 1. 本地环境初始化

克隆仓库后，在根目录安装项目依赖工具并激活 Git Hooks：

```sh
mise install
hk install --mise
```

激活后，以下 Git Hooks 会自动生效：

- **`pre-commit`**：运行文档规范、Mermaid 语法、离线死链、拼写检查、工作流安全检查，以及 `cargo fmt`、`ast-grep` 架构检查、`cargo clippy`、`cargo nextest`。
- **`commit-msg`**：校验提交信息是否符合 Conventional Commits 规范。
- **`pre-push`**：运行 `cargo deny check` 审计依赖安全与许可证合规。

## 2. 常用检查命令

- 执行全量只读静态检查、测试与依赖安全审计：

  ```sh
  mise run check
  ```

- 自动修复排版、Markdown 格式与代码格式：

  ```sh
  mise run fmt
  ```

## 3. 提交与版本规范

本仓库遵循 [Conventional Commits 1.0.0](https://www.conventionalcommits.org/) 与 [Semantic Versioning 2.0.0](https://semver.org/) 规范：

```text
<type>(<scope>): <动词开头的祈使短语，概括变更意图>

<可选正文：仅当变更缘由无法从代码差异自明时补充>
```

破坏性变更统一在作用域后标注感叹号 `<type>(<scope>)!:`。合入 `main` 的 `feat`、`fix` 与破坏性变更会自动触发 `release-please` 创建或更新 Release PR：在 `0.x` 阶段破坏性变更仅升级次版本号，`>= 1.0.0` 后升级主版本号；其余提交类型不触发发版。

## 4. Pull Request 流程

1. 从 `main` 分支创建特性或修复分支。
2. 提交前确认本地 `mise run check` 全部通过。
3. 创建 Pull Request，确保标题符合上述提交规范，等待 `Merge Gate` 通过后使用 `Squash and merge` 合并。

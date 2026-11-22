# 后端选用 SQLite WAL 并拆分 runtime 与 api

Status: accepted

持久化层统一采用 `SQLite` 配合 `WAL` 模式与 `sqlx` 迁移脚本；组合根按协议无关与协议宿主拆分为 `backend/runtime` 与 `backend/api`，命令行客户端默认通过 `local` 模式直连 `backend/runtime`，图形客户端通过 `remote` 模式对接 `backend/api`。

## 备选方案

- 生产使用 `PostgreSQL` 并为本地命令行另维护 `SQLite` 迁移：需要维护双套 SQL 方言且无法共用统一编译期校验。
- 将命令行与图形界面全部强绑定至同一远端服务：本地命令行使用必须先手动启动守护进程，削弱单二进制演示与独立运行能力。

## 影响

- `clients/gateway` 清空默认特性，由 `clients/cli` 启用 `local`、由 `clients/app` 启用 `remote`，保障网页端构建不引入服务端及原生数据库依赖。
- 全工作区共享同一套 `migrations/` 与 `sqlx::test` 隔离测试基础设施。

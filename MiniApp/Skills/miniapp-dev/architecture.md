# MiniApp 架构与源码入口

本文说明仓库内的职责与入口。应用作者使用的 API 和设计约定分别见
[产品内置 API 参考](../../../src/crates/assembly/core/builtin_skills/miniapp-dev/api-reference.md)和
[设计指南](../../../src/crates/assembly/core/builtin_skills/miniapp-dev/design-playbook.md)。

## 创建与更新流程

1. [`InitMiniApp`](../../../src/crates/assembly/core/src/agentic/tools/implementations/miniapp_init_tool.rs)
   创建应用骨架并返回 `app_id` 和应用目录。
2. Agent 使用通用文件工具修改返回目录中的 `source/index.html`、`source/style.css`、
   `source/ui.js` 及按需启用的 `source/worker.js` 等源码。
3. [`FinalizeMiniApp`](../../../src/crates/assembly/core/src/agentic/tools/implementations/miniapp_finalize_tool.rs)
   重新加载源码、编译并保存 `compiled.html`；内容变化时推进修订并发出运行时更新事件。
   应用版本字段由宿主维护，定制草稿使用其自身的同步和应用流程。
4. Web UI 的目录同步与运行器消费生命周期事件，刷新应用列表及已打开的应用。

UI 使用 ESM 和 `window.app`。Node Worker 通过 `app.call` 提供应用自定义方法；
可用宿主能力及权限以产品内置 API 说明、稳定契约和具体宿主实现为准。

## 模块职责

| 模块 | 所属职责 |
| --- | --- |
| [Product domains / MiniApp](../../../src/crates/contracts/product-domains/src/miniapp/) | DTO、权限策略、生命周期、Bridge、编译与运行时契约、内置应用资源 |
| [Integration services / MiniApp](../../../src/crates/services/services-integrations/src/miniapp/) | 具体存储、宿主调用、Worker 进程和运行时 IO |
| [Core / MiniApp](../../../src/crates/assembly/core/src/miniapp/) | 产品装配、Manager、宿主服务接入和兼容导出 |
| [Agent tools](../../../src/crates/assembly/core/src/agentic/tools/implementations/) | 应用初始化、修订发布和市场发布等工具入口 |
| [Web UI / MiniApps](../../../src/web-ui/src/app/scenes/miniapps/) | 场景、目录同步、Bridge 和 iframe 运行器 |
| [Desktop adapters](../../../src/apps/desktop/src/api/) | Desktop 命令与宿主适配；共享 UI 经基础设施层调用 |

UI、执行宿主与工作区可能位于不同机器。文件、进程和权限操作必须由负责该能力的宿主处理；
不可把控制端路径或本地服务作为远端操作的静默替代。修改命令或能力时，遵循根目录
[AGENTS.md](../../../AGENTS.md) 的远程场景与升级兼容规则。

## 文档与样例维护

产品内置 skill 必须能随应用独立分发，因此保留其自包含参考样例。
[MiniApp Demo](../../Demo/) 与内置样例目录的镜像关系由
[前端颜色审计配置](../../../scripts/frontend-color-surface-registry.json)声明，
由审计检查内容一致性。修改样例时同步对应副本。

维护实现时阅读所属模块最近的 `AGENTS.md`，运行与改动对应的最小验证。
本页不复制易过时的命令表、DTO 定义或生成资源。

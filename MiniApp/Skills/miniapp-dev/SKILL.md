---
name: miniapp-dev
description: Maintain MiniApp framework code in this repository, or find the canonical built-in MiniApp authoring guide and examples.
---

# MiniApp 开发入口

本目录保留仓库维护入口。产品分发的生成流程、API 和设计约定统一维护在
[内置 miniapp-dev skill](../../../src/crates/assembly/core/builtin_skills/miniapp-dev/SKILL.md)，
避免仓库指南与产品内置说明分别演进。

- 创建或修改 MiniApp：先读内置 skill，再查其
  [API 参考](../../../src/crates/assembly/core/builtin_skills/miniapp-dev/api-reference.md)和
  [设计指南](../../../src/crates/assembly/core/builtin_skills/miniapp-dev/design-playbook.md)。
- 修改框架：先读本目录的[架构与源码入口](architecture.md)，再遵循所属模块最近的 `AGENTS.md`。
- 查找完整样例：使用[内置参考样例](../../../src/crates/assembly/core/builtin_skills/miniapp-dev/references/examples/README.md)。
  仓库内的 [Demo](../../Demo/) 是对应示例的源目录，产品副本的一致性由主题审计校验。

生成流程以当前工具实现为准：`InitMiniApp` 创建骨架，通用文件工具修改源码，
`FinalizeMiniApp` 编译并发布新修订。定制草稿的同步和应用由定制宿主负责。
不要在本目录维护另一份 API 表、设计规则或产品运行时资源副本。

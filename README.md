# golutra

## English

### Tagline

One person. One AI squad.

### What is golutra

`golutra` is a next-generation multi-agent collaboration workspace.  
It goes beyond traditional IDE extensions and single-chat AI tools by orchestrating multiple agents to execute complex tasks in parallel.
Built with Vue 3 + Rust as a Tauri desktop application, `golutra` currently supports Windows and macOS.

### Core Value

- Unlimited multi-agent collaboration: automatically break down complex tasks and execute in parallel.
- Silent background execution: agents keep running even when you leave the main view.
- Terminal show/hide on demand: switch between high-level control and terminal details instantly.
- Avatar-based inspection: click each agent avatar to view terminal status, logs, and outputs.
- Automatic orchestration: from analysis and generation to strategy and deployment.
- Personal AI team: move from "one person + one editor" to "one person + an AI squad."

### CLI Compatibility Layer

Keep your existing CLI and instantly upgrade it into a collaboration hub.

No terminal switch, no workflow rewrite, no re-training. Keep using the CLI you already trust while adding multi-agent collaboration, orchestration, and result handoff to your current engineering pipeline.

Supported CLI tools:

- Claude Code
- Gemini CLI
- Codex CLI
- OpenCode
- Qwen Code

What you keep:

- No project migration
- No command relearning
- No single-tool lock-in

With your existing CLI, you can now:

- Run parallel multi-agent execution with automatic result handoff
- Track status and scheduling across CLIs in one orchestration layer
- Reuse session-level context and prompts without repeating requirements
- Aggregate test, build, and regression output into one delivery path

Keep your familiar commands. `golutra` wires them into a complete engineering loop.

### Stealth Terminal

Command line power, visual interface simplicity.

Seamlessly integrate code execution with a background terminal that adapts to your workflow. Experience the raw power of command-line control without leaving the visual interface.

- Direct Injection: inject prompts directly into the terminal stream for instant agent feedback loops.
- Context Awareness: the terminal understands your project context, offering intelligent autocompletion for complex tasks.

### Why Beyond Traditional IDEs

Traditional IDE workflows are usually "single-threaded + manual context switching."  
`golutra` is "parallel multi-agent execution + automated orchestration."

### Repository Scope

This repository is for source code storage and releases.

Business Email: [golutra&#64;hotmail.com](mailto:golutra%40hotmail.com)  
Official Website: [https://www.golutra.com/](https://www.golutra.com/)  
Video: <https://youtu.be/DKKracLotg8>  
Discord: [https://discord.gg/QyNVu56mpY](https://discord.gg/QyNVu56mpY)
Security Policy: See [SECURITY.md](SECURITY.md)

### Author

This software is independently developed and maintained by [seekskyworld](https://github.com/seekskyworld).

### Open Source Status

This project is now open source, and related code will be organized and opened in phases.

To protect security and privacy, any parts involving server keys, account configuration, or other sensitive information will be sanitized and refactored before being gradually published to the repository.

### Usage License

- Using `golutra` as a tool to build commercial software is allowed.
- Code and deliverables produced by users through `golutra` belong to the users.
- This project follows the [Business Source License 1.1 (BSL 1.1)](https://mariadb.com/bsl11/) open-source license.

### Downloads

- Releases: <https://github.com/golutra/golutra/releases>

### macOS Security Notice

When first installing from `golutra_macos_aarch64.dmg`, macOS may show:

- "App is damaged" or "Developer cannot be verified"

This is a macOS security mechanism and does not necessarily indicate an issue with the app.

How to open:

1. Open Terminal.
2. Run the command below (replace the path with your actual app location).
3. If installed to Applications, for example:

```bash
xattr -rd com.apple.quarantine /Applications/Golutra.app
```

Then open the app again.

Why this happens:

- macOS applies quarantine and signature checks to apps downloaded from the internet.
- Development or non-notarized builds may trigger this warning.
- This command only removes the quarantine flag and does not modify app contents.

If the app still cannot be opened, please contact the publisher for support.

## 中文

### 标语

一个人，拥有一个 AI 军团。

### 什么是 golutra

`golutra` 是新一代多 Agent 协同工作软件。  
它不是传统 IDE 的插件增强，也不是单一 AI 对话工具，而是把复杂任务交给多个智能 Agent 并行执行的系统级工作台。
`golutra` 基于 Vue 3 + Rust 开发，采用 Tauri 桌面应用架构，当前支持 Windows 和 macOS。

### 核心价值

- 无限 Agent 协作：自动拆解复杂任务，多个 Agent 并行推进。
- 后台静默运行：离开当前界面时，Agent 仍可持续执行任务。
- 终端随时显隐：主界面与终端状态快速切换，专注与透明兼得。
- 头像即入口：点击 Agent 头像即可查看各自终端状态、日志与结果输出。
- 自动编排流程：从分析、生成、策略到执行部署，形成完整闭环。
- 个人 AI 团队：从“一个人 + 一个编辑器”升级为“一个人 + 一支 AI 军团”。

### CLI 兼容生态

保留你原本的 CLI，也能一键升级成协作中枢。

不换终端、不改习惯、不重学流程。直接沿用你正在使用的 CLI，把多智能体协作、任务编排与结果回传能力接到现有工程链路里。

支持的 CLI 工具：

- Claude Code
- Gemini CLI
- Codex CLI
- OpenCode
- Qwen Code

你将保留：

- 无需迁移项目
- 无需重学命令
- 无需绑定单一工具

只用原本 CLI，也能做到：

- 多智能体并行执行，结果自动回传到同一工作流
- 跨 CLI 的统一调度与状态追踪，减少手动切换成本
- 会话级上下文记忆与指令复用，避免重复解释需求
- 测试、构建、回归信息集中汇总，交付路径更短

你继续使用熟悉的命令，`golutra` 负责把能力串联成完整工程闭环。

### 隐形终端

命令行级能力，可视化界面般易用。

将代码执行与后台终端无缝融合，让工作流实时响应。无需离开可视界面，也能获得命令行的原生掌控力。

- 直连注入：将提示词直接注入终端流，构建即时智能体反馈闭环。
- 上下文感知：终端理解项目上下文，可为复杂任务提供更智能的自动补全。

### 为什么超越传统 IDE

传统 IDE 工作流通常是“单线程 + 人工切换上下文”。  
`golutra` 是“多 Agent 并行 + 自动化编排协作”。

### 仓库说明

这个仓库用于源代码存放和版本发布。

商务邮箱: [golutra&#64;hotmail.com](mailto:golutra%40hotmail.com)  
官网: [https://www.golutra.com/](https://www.golutra.com/)  
视频地址: <https://www.bilibili.com/video/BV1qcfhBFEpP/?spm_id_from=333.1387.homepage.video_card.click>  
安全策略: 详见 [SECURITY.md](SECURITY.md)

### 作者

本软件由 [seekskyworld](https://github.com/seekskyworld) 独立开发与维护。

### 开源状态

项目现已开源，并将持续分阶段整理与开放相关代码。

为保障安全与隐私，涉及服务器密钥、账号配置及其他敏感信息的部分会先完成脱敏与重构，再逐步同步到仓库。

### 使用许可

- 允许将 `golutra` 作为工具用于商业软件开发。
- 用户通过 `golutra` 产出的代码与交付成果归用户所有。
- 项目遵守 [Business Source License 1.1 (BSL 1.1)](https://mariadb.com/bsl11/) 开源协议。

### 下载

- Releases: <https://github.com/golutra/golutra/releases>

### macOS 安全提示说明

首次通过 `golutra_macos_aarch64.dmg` 安装应用，打开时，macOS 可能会提示：

- “App 已损坏”或“无法验证开发者”

这是 macOS 的安全机制提示，并不一定代表应用本身存在问题。

打开方式：

1. 打开终端。
2. 输入以下命令（将路径替换为你的 App 实际位置）。
3. 例如安装后的位置：

```bash
xattr -rd com.apple.quarantine /Applications/Golutra.app
```

然后再次打开应用即可。

为什么会出现这个提示：

- macOS 会对来自互联网的应用进行隔离与签名校验。
- 开发版或未公证版本可能触发该提示。
- 该命令仅移除系统隔离标记，不会修改应用内容。

如果仍无法打开应用，请联系发布者获取帮助。

# golutra

golutra 是一款基于 Vue 3 + Tauri 的本地优先工作区应用。创建属于你的团队，轻松部署任务，并把项目与工作区绑定管理。

## 本地开发

**前置依赖:** Node.js, pnpm  
**桌面端额外依赖:** Rust + cargo-tauri

1. 安装依赖  
   `pnpm install`
2. 启动 Web 开发服务  
   `pnpm dev`
3. 启动桌面端开发（可选）  
   `cargo tauri dev`
4. 调试桌面端开发（可选）  
   $env:GOLUTRA_TERMINAL_TRACE="1"
   $env:GOLUTRA_TERMINAL_TRACE_DETAIL="1"
   $env:VITE_TERMINAL_TRACE="1"
   cargo tauri dev


## 清理 Rust / Cargo 缓存

在 src-tauri 或项目根目录执行：

cargo clean

这会删除：

target/  # 默认的 Rust 构建产物目录
注意：cargo clean 不会删除 Cargo.lock 文件或源代码，只会删除编译生成的二进制文件。

## 测试

`pnpm test`

## 代码检查

- `pnpm lint`
- `pnpm format:check`

## 目录结构

- `src/app/App.vue`: 应用壳与导航状态
- `src/features`: 功能模块（工作区、聊天、技能商店、插件、终端等）
- `src/shared`: 复用组件与 composables
- `src/i18n`: 文案与语言配置
- `src/styles/global.css`: 全局样式与工具类

See `docs/ENGINEERING_GUIDE.md` for conventions and structure details.

Legacy React code is moved to `C:\project\user\nexus-dashboard-suite-legacy`.

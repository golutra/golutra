//! 二进制入口：仅负责调用库层启动逻辑，保持可测试与可复用。
// 发行版 Windows 隐藏额外控制台窗口，避免 GUI 应用出现多余黑框。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // 保持入口极简，业务初始化统一在库层处理。
  app_lib::run();
}

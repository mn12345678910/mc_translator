// 在 Windows 的 Release 模式下防止顯示額外的控制台視窗，請勿刪除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run();
}

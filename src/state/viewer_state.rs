use std::sync::{Arc, RwLock, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use eframe::egui;

/// 子視窗更新訊息類型
pub enum ViewerUpdate {
    Theme(String),
    FontSize(f32),
    SaveConfig, // 新增：觸發存盤 (Revision 15.12)
}

/// 視窗共享狀態 (用於主子視窗同步)
pub struct ViewerSharedState {
    pub theme: Arc<RwLock<String>>,
    pub font_size: Arc<RwLock<f32>>,
    pub close_requested: Arc<std::sync::atomic::AtomicBool>,
    pub update_tx: UnboundedSender<ViewerUpdate>,
    pub opened_last_frame: Arc<Mutex<bool>>,
    /// 用於隱形啟動的幀數計數器
    pub opened_frames: Arc<Mutex<u32>>,
    // 視窗同步 (ses_342b)
    pub position: Arc<RwLock<Option<egui::Pos2>>>,
    pub inner_size: Arc<RwLock<Option<egui::Vec2>>>,
}

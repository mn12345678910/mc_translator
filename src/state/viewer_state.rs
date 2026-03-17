use std::sync::{Arc, RwLock, Mutex};
use tokio::sync::mpsc::UnboundedSender;
use eframe::egui;

/// 子視窗更新訊息類型
pub enum ViewerUpdate {
    Theme(String),
    FontSize(f32),
    Style(StyleSnapshot),
    SaveConfig,
}

/// 樣式快照，用於主子視窗間的完整視覺同步
#[derive(Clone, Debug)]
pub struct StyleSnapshot {
    pub dark_bg: [u8; 3],
    pub dark_text: [u8; 3],
    pub light_bg: [u8; 3],
    pub light_text: [u8; 3],
    pub dark_label: [u8; 3],
    pub light_label: [u8; 3],
    pub dark_btn_bg: [u8; 3],
    pub dark_btn_text: [u8; 3],
    pub light_btn_bg: [u8; 3],
    pub light_btn_text: [u8; 3],
    pub dark_input_bg: [u8; 3],
    pub light_input_bg: [u8; 3],
    pub dark_list_bg: [u8; 3],
    pub light_list_bg: [u8; 3],
    pub dark_tab_active: [u8; 3],
    pub light_tab_active: [u8; 3],
    pub rounding: f32,
    pub instance_overrides: std::collections::HashMap<String, crate::config::settings::ComponentStyle>,
}

/// 視窗共享狀態 (用於主子視窗同步)
pub struct ViewerSharedState {
    pub theme: Arc<RwLock<String>>,
    pub font_size: Arc<RwLock<f32>>,
    pub style: Arc<RwLock<StyleSnapshot>>,
    pub ui_lang: Arc<RwLock<String>>,
    pub close_requested: Arc<std::sync::atomic::AtomicBool>,
    pub update_tx: UnboundedSender<ViewerUpdate>,
    pub opened_last_frame: Arc<Mutex<bool>>,
    pub opened_frames: Arc<Mutex<u32>>,
    pub position: Arc<RwLock<Option<egui::Pos2>>>,
    pub inner_size: Arc<RwLock<Option<egui::Vec2>>>,
}

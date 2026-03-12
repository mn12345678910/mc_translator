use crate::translation::job::{JobConfig, JobSharedState};
use crate::state::app_state::AppState;
use crate::state::viewer_state::ViewerUpdate;
use crate::utils;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

impl AppState {
    pub fn refresh_all_dictionaries(&self) {
        Self::refresh_dictionaries_core(
            self.mc_lang.clone(),
            self.exact_match_map.clone(),
            self.inferred_match_map.clone(),
            self.term_replacements.clone(),
            self.status.clone(),
            self.runtime.handle().clone(),
            self.viewer_shared.update_tx.clone(),
            self.theme.clone(),
            self.show_memory_viewer,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn refresh_dictionaries_core(
        mc_lang_arc: Arc<Mutex<Option<utils::McLangFiles>>>,
        exact_arc: Arc<Mutex<HashMap<String, String>>>,
        inferred_arc: Arc<Mutex<HashMap<String, String>>>,
        term_arc: Arc<Mutex<Vec<(String, String)>>>,
        status_arc: Arc<Mutex<String>>,
        runtime_handle: tokio::runtime::Handle,
        viewer_update_tx: tokio::sync::mpsc::UnboundedSender<ViewerUpdate>,
        theme_clone: String,
        show_viewer_clone: bool,
    ) {
        *status_arc.lock().unwrap() = "正在分析辭典...".to_string();

        runtime_handle.spawn(async move {
            if let Ok((files, exact, unfiltered)) = crate::utils::load_mc_dicts().await {
                {
                    let mut exact_map = exact_arc.lock().unwrap();
                    *exact_map = exact.clone();
                }

                let inferred = crate::utils::analyze_dictionary(&exact);
                {
                    let mut inferred_map = inferred_arc.lock().unwrap();
                    *inferred_map = inferred.clone();
                }
                if show_viewer_clone {
                    let _ = viewer_update_tx
                        .send(ViewerUpdate::Theme(theme_clone));
                }

                {
                    let mut term_map = term_arc.lock().unwrap();
                    *term_map = unfiltered;
                }
                crate::config::save_dict(crate::config::OFFICIAL_DICT, &inferred);

                if let Ok(mut mc) = mc_lang_arc.lock() {
                    *mc = Some(files);
                }
            }
            *status_arc.lock().unwrap() = "就緒".to_string();
        });
    }

    pub fn resume_translation(&mut self) {
        if let Some(ref active_cfg) = self.active_job_config {
            let mut cfg = active_cfg.lock().unwrap();
            cfg.api_key = self.api_key.clone();
            cfg.api_provider = self.api_provider.clone();
            cfg.selected_model = self.selected_model.clone();
            cfg.ollama_url = self.ollama_url.clone();
            cfg.prompt = self.translation_prompt.clone();
            cfg.ollama_timeout = self.ollama_timeout as u64;
            cfg.batch_size = self.batch_size;
            cfg.batch_max_chars = self.batch_max_chars;
            cfg.pack_format = self.pack_format;
            cfg.glossary_priority = self.glossary_priority.lock().unwrap().clone();
            cfg.skip_json = self.skip_json;
            cfg.skip_js = self.skip_js;
            cfg.skip_jar = self.skip_jar;
            cfg.skip_book = self.skip_book;
            cfg.enable_llm_log = self.enable_llm_log;
        }

        *self.is_paused.lock().unwrap() = false;
        self.pause_notifier.notify_waiters();
        self.add_log(">>> 使用者恢復翻譯任務中...");
    }

    pub fn save_config(&self) {
        let mut config = crate::config::AppConfig::load();
        config.output_dir = self.output_dir.clone();
        config.provider = self.api_provider.clone();
        config.model = self.selected_model.clone();
        config.ollama_url = self.ollama_url.clone();
        config.batch_size = self.batch_size;
        config.batch_max_chars = self.batch_max_chars;
        config.ollama_timeout = self.ollama_timeout;
        config.translation_prompt = self.translation_prompt.clone();
        config.theme = self.theme.clone();
        config.pack_format = self.pack_format;
        config.font_size = self.font_size;
        config.glossary_priority = self.glossary_priority.lock().unwrap().clone();
        config.skip_json = self.skip_json;
        config.skip_js = self.skip_js;
        config.skip_jar = self.skip_jar;
        config.skip_book = self.skip_book;
        config.enable_llm_log = self.enable_llm_log;

        // --- 視窗幾幾何持久化 (Revision 15.17) ---
        config.viewer_x = self.viewer_x;
        config.viewer_y = self.viewer_y;
        config.viewer_width = self.viewer_width;
        config.viewer_height = self.viewer_height;
        config.main_x = self.main_x;
        config.main_y = self.main_y;
        config.main_width = self.main_width;
        config.main_height = self.main_height;

        if self.api_provider == "Ollama" {
            config.api_key = String::new();
        } else if self.api_provider != "DeepL" || !self.api_key.is_empty() {
            config.api_key = self.api_key.clone();
        }
        config.save();
    }

    pub fn refresh_models(&self) {
        let provider = self.api_provider.clone();
        let api_key = self.api_key.clone();
        let ollama_url = self.ollama_url.clone();
        let available_models = self.available_models.clone();

        self.runtime.spawn(async move {
            if let Ok(models) =
                crate::translation::fetch_dynamic_models(&provider, &api_key, &ollama_url).await
            {
                let mut m = available_models.lock().unwrap();
                *m = models;
            }
        });
    }

    pub fn refresh_mc_versions(&self) {
        let mc_versions = self.mc_versions.clone();
        self.runtime.spawn(async move {
            let versions = crate::translation::fetch_mc_versions().await;
            let mut v = mc_versions.lock().unwrap();
            *v = versions;
        });
    }

    pub fn start_translation(&mut self, _ctx: egui::Context) {
        if self.is_processing_active() {
            return;
        }

        *self.progress.lock().unwrap() = 0.0;
        *self.progress_total.lock().unwrap() = 0.0;
        *self.global_progress.lock().unwrap() = 0.0;
        *self.global_total.lock().unwrap() = 0.0;
        *self.status.lock().unwrap() = "正在分析檔案".to_string();

        self.add_log(">>> 開始翻譯任務...");

        let log = self.log.clone();
        let paths = self.input_paths.clone();
        let mc_lang_arc = self.mc_lang.clone();
        let term_arc = self.term_replacements.clone();
        let exact_arc = self.exact_match_map.clone();
        let processing_arc = self.is_processing.clone();
        let cancelled_arc = self.is_cancelled.clone();
        let paused_arc = self.is_paused.clone();
        let status_arc = self.status.clone();
        let translation_memory_arc = self.translation_memory.clone();

        let api_key = self.api_key.clone();
        let api_provider = self.api_provider.clone();
        let selected_model = self.selected_model.clone();
        let ollama_url = self.ollama_url.clone();
        let batch_size = self.batch_size;
        let batch_max_chars = self.batch_max_chars;
        let ollama_timeout = self.ollama_timeout;
        let translation_prompt = self.translation_prompt.clone();
        let output_dir_val = self.output_dir.clone();
        let pack_format_val = self.pack_format;

        let job_config = Arc::new(Mutex::new(JobConfig::new(
            api_key,
            api_provider,
            selected_model,
            ollama_url,
            translation_prompt,
            ollama_timeout,
            batch_size,
            batch_max_chars,
            output_dir_val,
            pack_format_val,
            self.glossary_priority.lock().unwrap().clone(),
            self.skip_json,
            self.skip_js,
            self.skip_jar,
            self.skip_book,
            self.enable_llm_log,
        )));
        self.active_job_config = Some(job_config.clone());

        let job_state = JobSharedState {
            log: log.clone(),
            status: status_arc.clone(),
            progress: self.progress.clone(),
            progress_total: self.progress_total.clone(),
            global_progress: self.global_progress.clone(),
            global_total: self.global_total.clone(),
            cancelled: self.is_cancelled.clone(),
            paused: paused_arc.clone(),
            translation_memory: translation_memory_arc.clone(),
            pause_notifier: self.pause_notifier.clone(),
            config: job_config.clone(),
        };

        *processing_arc.lock().unwrap() = true;
        *cancelled_arc.lock().unwrap() = false;
        *paused_arc.lock().unwrap() = false;

        let self_runtime = self.runtime.handle().clone();

        self_runtime.spawn(async move {
            let res =
                crate::file::pipeline::process_all_files(paths, job_state, mc_lang_arc, term_arc, exact_arc)
                    .await;

            *processing_arc.lock().unwrap() = false;
            let mut s = status_arc.lock().unwrap();
            if *cancelled_arc.lock().unwrap() {
                *s = "任務已取消".to_string();
                let mut l = log.lock().unwrap();
                l.push(">>> 任務已由使用者手動取消。".to_string());
            } else if let Err(e) = res {
                *s = format!("發生錯誤: {}", e);
                let mut l = log.lock().unwrap();
                l.push(format!(">>> 錯誤: {}", e));
            } else {
                *s = "任務完成".to_string();
                let mut l = log.lock().unwrap();
                l.push(">>> 所有翻譯任務已完成！".to_string());
            }
        });
    }

    pub fn stop_translation(&self) {
        *self.is_cancelled.lock().unwrap() = true;
        *self.is_paused.lock().unwrap() = false;
        self.pause_notifier.notify_waiters();
    }

    pub fn pause_translation(&self) {
        *self.is_paused.lock().unwrap() = true;
    }

    /// 啟動辭典目錄監控 (feat/dict-watcher)
    pub fn start_dict_watcher(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use notify::{Watcher, RecursiveMode, Config};
        use std::time::Duration;

        let exact_match_map = self.exact_match_map.clone();
        let inferred_match_map = self.inferred_match_map.clone();
        let term_replacements = self.term_replacements.clone();
        let status = self.status.clone();
        let runtime_handle = self.runtime.handle().clone();
        let viewer_update_tx = self.viewer_shared.update_tx.clone();
        let theme = self.theme.clone();
        let show_memory_viewer = self.show_memory_viewer;
        let mc_lang = self.mc_lang.clone();

        // 建立防抖 (Debounce) 的監控處理
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let mut watcher = notify::RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    // 1. 檢查事件類型：排除 Access (讀取) 事件，專注於 Modify, Create, Remove
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        // 2. 檢查檔案名稱：僅針對來源辭典檔案 (不包含 official.json 與 user.json)
                        let dict_files = ["en_us.json", "zh_cn.json", "zh_tw.json"];
                        let is_target = event.paths.iter().any(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| dict_files.contains(&s))
                                .unwrap_or(false)
                        });

                        if is_target {
                            let _ = tx.blocking_send(());
                        }
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        let dict_path = std::path::Path::new(crate::config::DICT_DIR);
        if !dict_path.exists() {
            let _ = std::fs::create_dir_all(dict_path);
        }
        watcher.watch(dict_path, RecursiveMode::NonRecursive)?;

        // 持久化 watcher
        self._dict_watcher = Some(Box::new(watcher));

        // 在非同步任務中處理事件並防抖
        let h_inner = runtime_handle.clone();
        runtime_handle.spawn(async move {
            while rx.recv().await.is_some() {
                // 簡單的防抖：收到訊號後等 500ms
                tokio::time::sleep(Duration::from_millis(500)).await;
                // 清空堆積的訊號
                while rx.try_recv().is_ok() {}

                Self::refresh_dictionaries_core(
                    mc_lang.clone(),
                    exact_match_map.clone(),
                    inferred_match_map.clone(),
                    term_replacements.clone(),
                    status.clone(),
                    h_inner.clone(),
                    viewer_update_tx.clone(),
                    theme.clone(),
                    show_memory_viewer,
                );
            }
        });

        Ok(())
    }
}

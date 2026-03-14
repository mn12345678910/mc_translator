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
            self.i18n.clone(),
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
        i18n: crate::ui::i18n::I18nLabels,
    ) {
        *status_arc.lock().unwrap() = i18n.status_analyzing_dict.clone();

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
            *status_arc.lock().unwrap() = i18n.status_ready.clone();
        });
    }

    pub fn resume_translation(&mut self) {
        if let Some(ref active_cfg) = self.active_job_config {
            let mut cfg = active_cfg.lock().unwrap();
            cfg.api_key = self.api_key.clone();
            cfg.api_provider = self.api_provider.clone();
            cfg.selected_model = self.selected_model.clone();
            cfg.ollama_url = self.ollama_url.clone();
            cfg.user_prompt = self.user_prompt.clone();
            cfg.system_prompt = self.system_prompt.clone();
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
            cfg.source_lang = self.source_lang.clone();
            cfg.target_lang = self.target_lang.clone();
        }

        *self.is_paused.lock().unwrap() = false;
        self.pause_notifier.notify_waiters();
        self.add_log(&self.i18n.log_resuming);
    }

    pub fn save_config(&self) {
        let packet = self.to_config_packet();
        packet.app.save();
        packet.style.save();
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
        self.trigger_save();

        if self.is_processing_active() {
            return;
        }

        let is_google_free = self.api_provider == "Google Free";
        if !is_google_free && self.selected_model.is_empty() {
             self.add_log(&self.i18n.log_start_failed.replace("{}", &self.api_provider));
             return;
        }

        self.add_log(&self.i18n.log_start_job
            .replace("{}", if self.api_provider.is_empty() { &self.i18n.status_not_ready } else { &self.api_provider })
            .replace("{}", if self.selected_model.is_empty() { "Google Free" } else { &self.selected_model })
        );

        *self.progress.lock().unwrap() = 0.0;
        *self.progress_total.lock().unwrap() = 0.0;
        *self.global_progress.lock().unwrap() = 0.0;
        *self.global_total.lock().unwrap() = 0.0;
        *self.status.lock().unwrap() = self.i18n.status_analyzing_files.to_string();

        let log = self.log.clone();
        let paths = self.input_paths.clone();
        let mc_lang_arc = self.mc_lang.clone();
        let term_arc = self.term_replacements.clone();
        let exact_arc = self.exact_match_map.clone();
        let inferred_arc = self.inferred_match_map.clone();
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
        let user_prompt = self.user_prompt.clone();
        let system_prompt = self.system_prompt.clone();
        let output_dir_val = self.output_dir.clone();
        let pack_format_val = self.pack_format;

        let job_config = Arc::new(Mutex::new(JobConfig::new(
            api_key,
            api_provider,
            selected_model,
            ollama_url,
            user_prompt,
            system_prompt,
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
            self.source_lang.clone(),
            self.target_lang.clone(),
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
            i18n: self.i18n.clone(),
        };

        *processing_arc.lock().unwrap() = true;
        *cancelled_arc.lock().unwrap() = false;
        *paused_arc.lock().unwrap() = false;

        let i18n = self.i18n.clone();

        tokio::spawn(async move {
            let res =
                crate::file::pipeline::process_all_files(
                    paths,
                    job_state,
                    mc_lang_arc,
                    term_arc,
                    exact_arc,
                    inferred_arc,
                )
                    .await;

            *processing_arc.lock().unwrap() = false;
            let mut s = status_arc.lock().unwrap();
            if *cancelled_arc.lock().unwrap() {
                *s = i18n.status_cancelled.to_string();
                let mut l = log.lock().unwrap();
                l.push(i18n.log_cancelled.to_string());
            } else if let Err(e) = res {
                *s = i18n.status_error.replace("{}", &e.to_string());
                let mut l = log.lock().unwrap();
                l.push(i18n.log_generic_error.replace("{}", &e.to_string()));
            } else {
                *s = i18n.status_finished.to_string();
                let mut l = log.lock().unwrap();
                l.push(i18n.log_finished.to_string());
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

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let mut watcher = notify::RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
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

        self._dict_watcher = Some(Box::new(watcher));

        let h_inner = runtime_handle.clone();
        runtime_handle.spawn(async move {
            while rx.recv().await.is_some() {
                tokio::time::sleep(Duration::from_millis(500)).await;
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
                    crate::ui::i18n::I18nLabels::load_or_default(crate::ui::i18n::DEFAULT_LANG), // 使用預設語言用於監控器更新，或從 AppState 傳遞
                );
            }
        });

        Ok(())
    }
}

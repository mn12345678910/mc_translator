use crate::config::settings::ComponentStyle;
use crate::config::{AppConfig, StyleConfig};
use crate::i18n::GuiLabels;
use crate::translation::LogSegment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum UiStatus {
    Idle,
    Running,
    Paused,
}

impl UiStatus {
    pub fn from_frontend_status(raw: &str) -> Self {
        match raw {
            "RUNNING" => Self::Running,
            "PAUSED" => Self::Paused,
            _ => Self::Idle,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UiPatch {
    pub status: String,
    pub show_translate: bool,
    pub show_pause: bool,
    pub show_resume: bool,
    pub show_stop: bool,
    pub lock_controls: bool,
    pub pause_notice: String,
    pub clear_current_status: bool,
    pub clear_batch_status: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PaletteState {
    pub show_property_group: bool,
    pub show_clear_group: bool,
    pub show_color_group: bool,
    pub show_number_group: bool,
    pub label_palette_number: String,
    pub label_palette_color: String,
    pub number_value: f32,
    pub number_step: f32,
    pub color_value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PanelState {
    pub show_api_settings: bool,
    pub show_developer_mode: bool,
    pub show_palette_settings: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigUiState {
    pub show_ollama_url: bool,
    pub show_api_key: bool,
    pub show_api_base_url: bool,
    pub show_fast_convert: bool,
    pub can_translate: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DictionaryPageData {
    pub items: Vec<(String, String)>,
    pub total_pages: usize,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuiInitState {
    pub config: AppConfig,
    pub style: StyleConfig,
    pub labels: GuiLabels,
    pub css_vars: HashMap<String, String>,
    pub ui_patch: UiPatch,
    pub toggle_labels: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PaletteInput {
    pub target_type: String,
    pub target_item: String,
    pub property: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigFormInput {
    pub api_provider: String,
    pub api_base_url: String,
    pub ollama_url: String,
    pub model: String,
    pub source_lang: String,
    pub target_lang: String,
    pub batch_size: String,
    pub batch_max_chars: String,
    pub timeout: String,
    pub output_dir: String,
    pub pack_format: String,
    pub user_prompt: String,
    pub system_prompt: String,
    pub glossary_priority: String,
    pub skip_json: bool,
    pub skip_js: bool,
    pub skip_jar: bool,
    pub skip_book: bool,
    pub enable_llm_log: bool,
    pub enable_debug_log: bool,
    pub show_debug_tools: bool,
    pub ui_lang: String,
    pub path: String,
    pub fast_convert: bool,
    pub excluded_paths_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StyleFormInput {
    pub font_size: String,
    pub btn_rounding_enabled: bool,
    pub btn_rounding_value: String,
    pub progress_pulse_enabled: bool,
    pub progress_pulse_speed: String,
    pub progress_style: String,
    pub color_bg: Option<String>,
    pub color_text: Option<String>,
    pub color_accent: Option<String>,
    pub color_danger: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PaletteMutationInput {
    pub target_type: String,
    pub target_item: String,
    pub property: String,
    pub color_hex: Option<String>,
    pub number_value: Option<f32>,
}

pub fn derive_ui_patch(status: UiStatus, labels: &GuiLabels) -> UiPatch {
    match status {
        UiStatus::Idle => UiPatch {
            status: "IDLE".to_string(),
            show_translate: true,
            show_pause: false,
            show_resume: false,
            show_stop: false,
            lock_controls: false,
            pause_notice: String::new(),
            clear_current_status: true,
            clear_batch_status: true,
        },
        UiStatus::Running => UiPatch {
            status: "RUNNING".to_string(),
            show_translate: false,
            show_pause: true,
            show_resume: false,
            show_stop: false,
            lock_controls: true,
            pause_notice: String::new(),
            clear_current_status: false,
            clear_batch_status: false,
        },
        UiStatus::Paused => UiPatch {
            status: "PAUSED".to_string(),
            show_translate: false,
            show_pause: false,
            show_resume: true,
            show_stop: true,
            lock_controls: false,
            pause_notice: labels
                .common
                .status_trans_paused
                .replace("{}", "")
                .trim()
                .to_string(),
            clear_current_status: false,
            clear_batch_status: false,
        },
    }
}

pub fn derive_toggle_labels(config: &AppConfig, labels: &GuiLabels) -> HashMap<String, String> {
    let mut out = HashMap::new();

    out.insert(
        "chk-glossary-priority".to_string(),
        if config.glossary_priority == "user" {
            labels.glossary_priority_user.clone()
        } else {
            labels.glossary_priority_official.clone()
        },
    );
    out.insert(
        "chk-llm-log".to_string(),
        if config.enable_llm_log {
            labels.label_enable_log.clone()
        } else {
            labels.label_disable_log.clone()
        },
    );
    out.insert(
        "chk-skip-json".to_string(),
        if config.skip_json {
            labels.label_skip_json.clone()
        } else {
            labels.label_no_skip_json.clone()
        },
    );
    out.insert(
        "chk-skip-js".to_string(),
        if config.skip_js {
            labels.label_skip_js.clone()
        } else {
            labels.label_no_skip_js.clone()
        },
    );
    out.insert(
        "chk-skip-jar".to_string(),
        if config.skip_jar {
            labels.label_skip_jar.clone()
        } else {
            labels.label_no_skip_jar.clone()
        },
    );
    out.insert(
        "chk-skip-book".to_string(),
        if config.skip_book {
            labels.label_skip_book.clone()
        } else {
            labels.label_no_skip_book.clone()
        },
    );
    out.insert(
        "chk-debug-log".to_string(),
        if config.enable_debug_log {
            labels.label_enable_debug_log.clone()
        } else {
            labels.label_disable_debug_log.clone()
        },
    );
    out.insert(
        "chk-debug-tools".to_string(),
        if config.show_debug_tools {
            labels.label_hide_debug_tools.clone()
        } else {
            labels.label_show_debug_tools.clone()
        },
    );
    out.insert(
        "chk-fast-convert".to_string(),
        if config.fast_convert {
            labels.label_fast_convert_on.clone()
        } else {
            labels.label_fast_convert_off.clone()
        },
    );

    out
}

pub fn derive_css_vars(style: &StyleConfig) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let is_dark = style.theme != "light";

    let bg = if is_dark {
        style.dark_bg
    } else {
        style.light_bg
    };
    let text = if is_dark {
        style.dark_text
    } else {
        style.light_text
    };
    let accent = if is_dark {
        style.dark_accent
    } else {
        style.light_accent
    };
    let danger = if is_dark {
        style.dark_danger
    } else {
        style.light_danger
    };

    out.insert("--bg-color".to_string(), rgb(bg));
    out.insert("--text-color".to_string(), rgb(text));
    out.insert("--accent-color".to_string(), rgb(accent));
    out.insert(
        "--accent-color-rgb".to_string(),
        format!("{}, {}, {}", accent[0], accent[1], accent[2]),
    );
    out.insert("--danger-color".to_string(), rgb(danger));
    out.insert("--font-size".to_string(), format!("{}px", style.font_size));
    out.insert(
        "--border-radius".to_string(),
        if style.btn_rounding_enabled {
            format!("{}px", style.btn_rounding_value)
        } else {
            "0px".to_string()
        },
    );

    let border = if is_dark {
        style.dark_border_color
    } else {
        style.light_border_color
    };
    out.insert("--border-color".to_string(), rgb(border));
    out.insert(
        "--border-light".to_string(),
        rgba(border, style.border_alpha),
    );

    let input_bg = if is_dark {
        style.dark_input_bg
    } else {
        style.light_input_bg
    };
    let btn_bg = if is_dark {
        style.dark_btn_bg
    } else {
        style.light_btn_bg
    };
    let btn_text = if is_dark {
        style.dark_btn_text
    } else {
        style.light_btn_text
    };
    let list_bg = if is_dark {
        style.dark_list_bg
    } else {
        style.light_list_bg
    };
    let text_muted = if is_dark {
        style.dark_text_muted
    } else {
        style.light_text_muted
    };
    let progress_bg = if is_dark {
        style.dark_progress_bg
    } else {
        style.light_progress_bg
    };
    out.insert("--input-bg".to_string(), rgb(input_bg));
    out.insert("--btn-bg".to_string(), rgb(btn_bg));
    out.insert("--btn-text".to_string(), rgb(btn_text));
    out.insert("--list-bg".to_string(), rgb(list_bg));
    out.insert("--text-muted".to_string(), rgb(text_muted));
    out.insert("--progress-bg".to_string(), rgb(progress_bg));
    out.insert("--space-sm".to_string(), format!("{}px", style.space_sm));
    out.insert("--space-md".to_string(), format!("{}px", style.space_md));
    out.insert("--space-lg".to_string(), format!("{}px", style.space_lg));

    out
}

pub fn derive_palette_state(
    style: &StyleConfig,
    labels: &GuiLabels,
    input: PaletteInput,
) -> PaletteState {
    let is_specific = input.target_type == "specific";
    let target = input.target_item;

    let is_number_item = target.starts_with("space_")
        || target.ends_with("_alpha")
        || target == "font_size"
        || (is_specific && input.property == "rounding");
    let is_log_color = target.contains("log_");
    let final_is_number = is_number_item && !is_log_color;

    let number_label = if target.starts_with("space_") {
        labels.palette_label_spacing.clone()
    } else if target.ends_with("_alpha") {
        labels.palette_label_alpha.clone()
    } else if target == "font_size" {
        labels.label_font_size.clone()
    } else {
        labels.palette_label_rounding.clone()
    };

    let number_value = if is_specific {
        style
            .instance_overrides
            .get(&target)
            .and_then(|ov| ov.rounding)
            .unwrap_or(4.0)
    } else if target == "font_size" {
        style.font_size
    } else if target == "space_sm" {
        style.space_sm
    } else if target == "space_md" {
        style.space_md
    } else if target == "space_lg" {
        style.space_lg
    } else if target == "border_alpha" {
        style.border_alpha
    } else if target == "panel_alpha" {
        style.panel_alpha
    } else if target == "backdrop_alpha" {
        style.backdrop_alpha
    } else {
        style.btn_rounding_value
    };

    let is_dark = style.theme != "light";
    let color_value = if final_is_number {
        "#ffffff".to_string()
    } else if is_specific {
        let override_cfg = style.instance_overrides.get(&target);
        let color = if input.property == "text" {
            if is_dark {
                override_cfg.and_then(|ov| ov.dark_text)
            } else {
                override_cfg.and_then(|ov| ov.light_text)
            }
        } else if is_dark {
            override_cfg.and_then(|ov| ov.dark_bg)
        } else {
            override_cfg.and_then(|ov| ov.light_bg)
        };
        color.map(hex).unwrap_or_else(|| "#ffffff".to_string())
    } else {
        pick_global_color(style, &target, is_dark)
            .map(hex)
            .unwrap_or_else(|| "#ffffff".to_string())
    };

    PaletteState {
        show_property_group: is_specific,
        show_clear_group: is_specific,
        show_color_group: !final_is_number,
        show_number_group: final_is_number,
        label_palette_number: number_label,
        label_palette_color: if input.property == "bg" {
            labels.label_bg_color.clone()
        } else {
            labels.label_text_color.clone()
        },
        number_value,
        number_step: if target.ends_with("_alpha") {
            0.01
        } else {
            1.0
        },
        color_value,
    }
}

pub fn build_gui_init_state(
    config: AppConfig,
    style: StyleConfig,
    labels: GuiLabels,
) -> GuiInitState {
    let css_vars = derive_css_vars(&style);
    let ui_patch = derive_ui_patch(UiStatus::Idle, &labels);
    let toggle_labels = derive_toggle_labels(&config, &labels);

    GuiInitState {
        config,
        style,
        labels,
        css_vars,
        ui_patch,
        toggle_labels,
    }
}

pub fn derive_config_ui_state(
    provider: &str,
    selected_model: &str,
    api_key: &str,
    source_lang: &str,
    target_lang: &str,
) -> ConfigUiState {
    let no_key_providers = ["Ollama", "Google Free", "無"];
    let hide_key = no_key_providers.contains(&provider);
    let can_skip_model = provider == "Google Free" || provider == "Ollama";
    let has_key = if hide_key {
        true
    } else {
        !api_key.trim().is_empty()
    };
    let has_model = can_skip_model || !selected_model.trim().is_empty();

    ConfigUiState {
        show_ollama_url: provider == "Ollama",
        show_api_key: !hide_key,
        show_api_base_url: !hide_key,
        show_fast_convert: (target_lang == "zh_cn" || target_lang == "zh_tw")
            && source_lang != target_lang,
        can_translate: has_key && has_model,
    }
}

pub fn derive_panel_state(action: &str, current: PanelState) -> PanelState {
    let mut next = current;
    match action {
        "toggle_api" => {
            next.show_api_settings = !next.show_api_settings;
            if next.show_api_settings {
                next.show_developer_mode = false;
                next.show_palette_settings = false;
            }
        }
        "toggle_dev" => {
            next.show_developer_mode = !next.show_developer_mode;
            if next.show_developer_mode {
                next.show_api_settings = false;
                next.show_palette_settings = false;
            }
        }
        "toggle_palette" => {
            next.show_palette_settings = !next.show_palette_settings;
            if next.show_palette_settings {
                next.show_api_settings = false;
                next.show_developer_mode = false;
            }
        }
        _ => {}
    }
    next
}

pub fn normalize_form_config(mut config: AppConfig) -> AppConfig {
    config.validate();
    config
}

pub fn build_form_config(mut base: AppConfig, input: ConfigFormInput) -> AppConfig {
    base.api_provider = input.api_provider;
    base.api_base_url = input.api_base_url;
    base.ollama_url = input.ollama_url;
    base.model = input.model;
    base.source_lang = input.source_lang;
    base.target_lang = input.target_lang;
    base.batch_size = parse_u32_or(&input.batch_size, base.batch_size);
    base.batch_max_chars = parse_u32_or(&input.batch_max_chars, base.batch_max_chars);
    base.timeout = parse_u32_or(&input.timeout, base.timeout);
    base.output_dir = input.output_dir;
    base.pack_format = parse_u32_or(&input.pack_format, base.pack_format);
    base.user_prompt = input.user_prompt;
    base.system_prompt = input.system_prompt;
    base.glossary_priority = input.glossary_priority;
    base.skip_json = input.skip_json;
    base.skip_js = input.skip_js;
    base.skip_jar = input.skip_jar;
    base.skip_book = input.skip_book;
    base.enable_llm_log = input.enable_llm_log;
    base.enable_debug_log = input.enable_debug_log;
    base.show_debug_tools = input.show_debug_tools;
    base.ui_lang = input.ui_lang;
    base.fast_convert = input.fast_convert;
    base.excluded_paths = input
        .excluded_paths_text
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect();
    base.validate();
    base
}

pub fn build_style_from_form(mut base: StyleConfig, input: StyleFormInput) -> StyleConfig {
    base.font_size = parse_f32_or(&input.font_size, base.font_size);
    base.btn_rounding_enabled = input.btn_rounding_enabled;
    base.btn_rounding_value = parse_f32_or(&input.btn_rounding_value, base.btn_rounding_value);
    base.progress_pulse_enabled = input.progress_pulse_enabled;
    base.progress_pulse_speed =
        parse_f32_or(&input.progress_pulse_speed, base.progress_pulse_speed);
    if !input.progress_style.trim().is_empty() {
        base.progress_style = input.progress_style;
    }

    if let Some(hex) = input.color_bg.as_deref().and_then(parse_hex_color) {
        base.dark_bg = hex;
    }
    if let Some(hex) = input.color_text.as_deref().and_then(parse_hex_color) {
        base.dark_text = hex;
    }
    if let Some(hex) = input.color_accent.as_deref().and_then(parse_hex_color) {
        base.dark_accent = hex;
    }
    if let Some(hex) = input.color_danger.as_deref().and_then(parse_hex_color) {
        base.dark_danger = hex;
    }
    base.validate();
    base
}

pub fn toggle_style_theme(mut style: StyleConfig) -> StyleConfig {
    style.theme = if style.theme == "dark" {
        "light".to_string()
    } else {
        "dark".to_string()
    };
    style.validate();
    style
}

pub fn apply_palette_mutation(mut style: StyleConfig, input: PaletteMutationInput) -> StyleConfig {
    let is_specific = input.target_type == "specific";
    let mut target = input.target_item;
    let is_dark = style.theme != "light";

    if let Some(rgb) = input.color_hex.as_deref().and_then(parse_hex_color) {
        if !is_specific {
            if target.starts_with("dark_") || target.starts_with("light_") {
                let base_key = target
                    .split_once('_')
                    .map(|(_, key)| key)
                    .unwrap_or(target.as_str());
                target = format!("{}_{}", if is_dark { "dark" } else { "light" }, base_key);
            }
            let _ = set_global_color(&mut style, &target, rgb);
        } else {
            let entry = style.instance_overrides.entry(target.clone()).or_default();
            let prop = format!(
                "{}_{}",
                if is_dark { "dark" } else { "light" },
                input.property
            );
            set_component_color(entry, &prop, rgb);
        }
    }

    if let Some(number) = input.number_value {
        if is_specific {
            let entry = style.instance_overrides.entry(target.clone()).or_default();
            entry.rounding = Some(number);
        } else {
            let _ = set_global_number(&mut style, &target, number);
        }
    }

    style.validate();
    style
}

pub fn clear_palette_override(mut style: StyleConfig, target: &str) -> StyleConfig {
    style.instance_overrides.remove(target);
    style
}

pub fn parse_log_segments(message: &str) -> Vec<LogSegment> {
    let mut segments = Vec::new();
    let mut rest = message;

    while let Some(start) = rest.find('<') {
        if start > 0 {
            segments.push(LogSegment {
                kind: "text".to_string(),
                text: rest[..start].to_string(),
            });
        }
        if rest[start..].starts_with("<dir>") {
            if let Some(end_idx) = rest[start + 5..].find("</dir>") {
                let content = &rest[start + 5..start + 5 + end_idx];
                segments.push(LogSegment {
                    kind: "dir".to_string(),
                    text: content.to_string(),
                });
                rest = &rest[start + 5 + end_idx + 6..];
                continue;
            }
        }
        if rest[start..].starts_with("<file>") {
            if let Some(end_idx) = rest[start + 6..].find("</file>") {
                let content = &rest[start + 6..start + 6 + end_idx];
                segments.push(LogSegment {
                    kind: "file".to_string(),
                    text: content.to_string(),
                });
                rest = &rest[start + 6 + end_idx + 7..];
                continue;
            }
        }
        segments.push(LogSegment {
            kind: "text".to_string(),
            text: rest[start..start + 1].to_string(),
        });
        rest = &rest[start + 1..];
    }

    if !rest.is_empty() {
        segments.push(LogSegment {
            kind: "text".to_string(),
            text: rest.to_string(),
        });
    }

    if segments.is_empty() {
        segments.push(LogSegment {
            kind: "text".to_string(),
            text: String::new(),
        });
    }
    segments
}

fn rgb(color: [u8; 3]) -> String {
    format!("rgb({},{},{})", color[0], color[1], color[2])
}

fn rgba(color: [u8; 3], alpha: f32) -> String {
    format!("rgba({},{},{},{})", color[0], color[1], color[2], alpha)
}

fn hex(color: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2])
}

fn pick_global_color(style: &StyleConfig, target: &str, is_dark: bool) -> Option<[u8; 3]> {
    match target {
        "dark_bg" | "light_bg" => Some(if is_dark {
            style.dark_bg
        } else {
            style.light_bg
        }),
        "dark_text" | "light_text" => Some(if is_dark {
            style.dark_text
        } else {
            style.light_text
        }),
        "dark_accent" | "light_accent" => Some(if is_dark {
            style.dark_accent
        } else {
            style.light_accent
        }),
        "dark_danger" | "light_danger" => Some(if is_dark {
            style.dark_danger
        } else {
            style.light_danger
        }),
        "dark_label" | "light_label" => Some(if is_dark {
            style.dark_label
        } else {
            style.light_label
        }),
        "dark_text_muted" | "light_text_muted" => Some(if is_dark {
            style.dark_text_muted
        } else {
            style.light_text_muted
        }),
        "dark_header_bg" | "light_header_bg" => Some(if is_dark {
            style.dark_header_bg
        } else {
            style.light_header_bg
        }),
        "dark_btn_bg" | "light_btn_bg" => Some(if is_dark {
            style.dark_btn_bg
        } else {
            style.light_btn_bg
        }),
        "dark_btn_text" | "light_btn_text" => Some(if is_dark {
            style.dark_btn_text
        } else {
            style.light_btn_text
        }),
        "dark_input_bg" | "light_input_bg" => Some(if is_dark {
            style.dark_input_bg
        } else {
            style.light_input_bg
        }),
        "dark_list_bg" | "light_list_bg" => Some(if is_dark {
            style.dark_list_bg
        } else {
            style.light_list_bg
        }),
        "dark_tab_active" | "light_tab_active" => Some(if is_dark {
            style.dark_tab_active
        } else {
            style.light_tab_active
        }),
        "dark_tab_inactive" | "light_tab_inactive" => Some(if is_dark {
            style.dark_tab_inactive
        } else {
            style.light_tab_inactive
        }),
        "dark_border_color" | "light_border_color" => Some(if is_dark {
            style.dark_border_color
        } else {
            style.light_border_color
        }),
        "dark_hover_bg" | "light_hover_bg" => Some(if is_dark {
            style.dark_hover_bg
        } else {
            style.light_hover_bg
        }),
        "dark_slider_bg" | "light_slider_bg" => Some(if is_dark {
            style.dark_slider_bg
        } else {
            style.light_slider_bg
        }),
        "dark_slider_thumb" | "light_slider_thumb" => Some(if is_dark {
            style.dark_slider_thumb
        } else {
            style.light_slider_thumb
        }),
        "dark_switch_bg" | "light_switch_bg" => Some(if is_dark {
            style.dark_switch_bg
        } else {
            style.light_switch_bg
        }),
        "dark_progress_bg" | "light_progress_bg" => Some(if is_dark {
            style.dark_progress_bg
        } else {
            style.light_progress_bg
        }),
        "dark_log_info" | "light_log_info" => Some(if is_dark {
            style.dark_log_info
        } else {
            style.light_log_info
        }),
        "dark_log_warn" | "light_log_warn" => Some(if is_dark {
            style.dark_log_warn
        } else {
            style.light_log_warn
        }),
        "dark_log_error" | "light_log_error" => Some(if is_dark {
            style.dark_log_error
        } else {
            style.light_log_error
        }),
        "dark_log_success" | "light_log_success" => Some(if is_dark {
            style.dark_log_success
        } else {
            style.light_log_success
        }),
        "dark_log_dir" | "light_log_dir" => Some(if is_dark {
            style.dark_log_dir
        } else {
            style.light_log_dir
        }),
        "dark_log_file" | "light_log_file" => Some(if is_dark {
            style.dark_log_file
        } else {
            style.light_log_file
        }),
        "aurora_1" => Some(style.aurora_1),
        "aurora_2" => Some(style.aurora_2),
        "aurora_3" => Some(style.aurora_3),
        "neon_color" => Some(style.neon_color),
        _ => None,
    }
}

fn parse_u32_or(raw: &str, fallback: u32) -> u32 {
    raw.trim().parse::<u32>().unwrap_or(fallback)
}

fn parse_f32_or(raw: &str, fallback: f32) -> f32 {
    raw.trim().parse::<f32>().unwrap_or(fallback)
}

fn parse_hex_color(raw: &str) -> Option<[u8; 3]> {
    let hex = raw.trim();
    if hex.len() != 7 || !hex.starts_with('#') {
        return None;
    }
    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
    Some([r, g, b])
}

fn set_component_color(component: &mut ComponentStyle, prop: &str, rgb: [u8; 3]) {
    match prop {
        "dark_bg" => component.dark_bg = Some(rgb),
        "dark_text" => component.dark_text = Some(rgb),
        "light_bg" => component.light_bg = Some(rgb),
        "light_text" => component.light_text = Some(rgb),
        _ => {}
    }
}

fn set_global_color(style: &mut StyleConfig, target: &str, rgb: [u8; 3]) -> bool {
    match target {
        "dark_bg" => style.dark_bg = rgb,
        "light_bg" => style.light_bg = rgb,
        "dark_text" => style.dark_text = rgb,
        "light_text" => style.light_text = rgb,
        "dark_accent" => style.dark_accent = rgb,
        "light_accent" => style.light_accent = rgb,
        "dark_danger" => style.dark_danger = rgb,
        "light_danger" => style.light_danger = rgb,
        "dark_label" => style.dark_label = rgb,
        "light_label" => style.light_label = rgb,
        "dark_text_muted" => style.dark_text_muted = rgb,
        "light_text_muted" => style.light_text_muted = rgb,
        "dark_header_bg" => style.dark_header_bg = rgb,
        "light_header_bg" => style.light_header_bg = rgb,
        "dark_btn_bg" => style.dark_btn_bg = rgb,
        "light_btn_bg" => style.light_btn_bg = rgb,
        "dark_btn_text" => style.dark_btn_text = rgb,
        "light_btn_text" => style.light_btn_text = rgb,
        "dark_input_bg" => style.dark_input_bg = rgb,
        "light_input_bg" => style.light_input_bg = rgb,
        "dark_list_bg" => style.dark_list_bg = rgb,
        "light_list_bg" => style.light_list_bg = rgb,
        "dark_tab_active" => style.dark_tab_active = rgb,
        "light_tab_active" => style.light_tab_active = rgb,
        "dark_tab_inactive" => style.dark_tab_inactive = rgb,
        "light_tab_inactive" => style.light_tab_inactive = rgb,
        "dark_border_color" => style.dark_border_color = rgb,
        "light_border_color" => style.light_border_color = rgb,
        "dark_hover_bg" => style.dark_hover_bg = rgb,
        "light_hover_bg" => style.light_hover_bg = rgb,
        "dark_slider_bg" => style.dark_slider_bg = rgb,
        "light_slider_bg" => style.light_slider_bg = rgb,
        "dark_slider_thumb" => style.dark_slider_thumb = rgb,
        "light_slider_thumb" => style.light_slider_thumb = rgb,
        "dark_switch_bg" => style.dark_switch_bg = rgb,
        "light_switch_bg" => style.light_switch_bg = rgb,
        "dark_progress_bg" => style.dark_progress_bg = rgb,
        "light_progress_bg" => style.light_progress_bg = rgb,
        "dark_log_info" => style.dark_log_info = rgb,
        "light_log_info" => style.light_log_info = rgb,
        "dark_log_warn" => style.dark_log_warn = rgb,
        "light_log_warn" => style.light_log_warn = rgb,
        "dark_log_error" => style.dark_log_error = rgb,
        "light_log_error" => style.light_log_error = rgb,
        "dark_log_success" => style.dark_log_success = rgb,
        "light_log_success" => style.light_log_success = rgb,
        "dark_log_dir" => style.dark_log_dir = rgb,
        "light_log_dir" => style.light_log_dir = rgb,
        "dark_log_file" => style.dark_log_file = rgb,
        "light_log_file" => style.light_log_file = rgb,
        "aurora_1" => style.aurora_1 = rgb,
        "aurora_2" => style.aurora_2 = rgb,
        "aurora_3" => style.aurora_3 = rgb,
        "neon_color" => style.neon_color = rgb,
        _ => return false,
    }
    true
}

fn set_global_number(style: &mut StyleConfig, target: &str, value: f32) -> bool {
    match target {
        "font_size" => style.font_size = value,
        "space_sm" => style.space_sm = value,
        "space_md" => style.space_md = value,
        "space_lg" => style.space_lg = value,
        "border_alpha" => style.border_alpha = value,
        "panel_alpha" => style.panel_alpha = value,
        "backdrop_alpha" => style.backdrop_alpha = value,
        "btn_rounding_value" => style.btn_rounding_value = value,
        "progress_pulse_speed" => style.progress_pulse_speed = value,
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_ui_patch_idle() {
        let labels = GuiLabels::default_zh_tw();
        let patch = derive_ui_patch(UiStatus::Idle, &labels);
        assert_eq!(patch.status, "IDLE");
        assert!(patch.show_translate);
        assert!(!patch.lock_controls);
    }

    #[test]
    fn derive_css_vars_has_basics() {
        let style = StyleConfig::default();
        let vars = derive_css_vars(&style);
        assert!(vars.contains_key("--bg-color"));
        assert!(vars.contains_key("--accent-color"));
        assert!(vars.contains_key("--font-size"));
    }

    #[test]
    fn apply_palette_mutation_updates_global_color() {
        let style = StyleConfig::default();
        let next = apply_palette_mutation(
            style,
            PaletteMutationInput {
                target_type: "global".to_string(),
                target_item: "dark_bg".to_string(),
                property: "bg".to_string(),
                color_hex: Some("#112233".to_string()),
                number_value: None,
            },
        );
        assert_eq!(next.dark_bg, [17, 34, 51]);
    }
}

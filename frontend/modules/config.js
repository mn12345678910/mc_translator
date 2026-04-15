// frontend/modules/config.js
import { state } from './state.js';
import { appendLog } from './utils.js';
import { dom } from './dom.js';

// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);

function applyConfigUiState(ui) {
    if (!ui || typeof ui !== 'object') return;

    if (dom.ollamaUrlGroup) {
        dom.ollamaUrlGroup.style.display = ui.show_ollama_url ? 'block' : 'none';
    }
    if (dom.apiKeyGroup) {
        dom.apiKeyGroup.style.display = ui.show_api_key ? 'block' : 'none';
    }
    if (dom.apiBaseUrlGroup) {
        dom.apiBaseUrlGroup.style.display = ui.show_api_base_url ? 'block' : 'none';
    }
    if (dom.groupFastConvert) {
        dom.groupFastConvert.style.display = ui.show_fast_convert ? 'block' : 'none';
    }
    if (dom.btnTranslate) {
        dom.btnTranslate.disabled = !ui.can_translate;
    }
}

export function refreshConfigUiState() {
    Promise.resolve(
        invoke('derive_config_ui_state_cmd', {
            provider: dom.apiProvider ? dom.apiProvider.value : '無',
            selectedModel: dom.selectedModel ? dom.selectedModel.value : '',
            apiKey: dom.apiKey ? dom.apiKey.value : '',
            sourceLang: dom.sourceLang ? dom.sourceLang.value : 'en_us',
            targetLang: dom.targetLang ? dom.targetLang.value : 'zh_tw',
        })
    ).then((ui) => applyConfigUiState(ui));
}

export async function loadConfig() {
    try {
        const config = await invoke('get_config');

        // [Vite 偵錯優化] 在開發環境下預設開啟開發者模式與偵錯工具
        if (typeof import.meta !== 'undefined' && import.meta.env?.DEV) {
            config.show_developer_mode = true;
            config.show_debug_tools = true;
            console.log('[DEBUG] Vite 環境偵測：已自動開啟開發者模式與偵錯工具');
        }

        state.currentConfig = config;

        if (dom.apiProvider) dom.apiProvider.value = config.api_provider;

        const savedKey = await invoke('get_api_key_cmd');
        if (dom.apiKey) dom.apiKey.value = savedKey || '';

        if (dom.apiBaseUrl) dom.apiBaseUrl.value = config.api_base_url || '';

        if (dom.inputPath) dom.inputPath.value = config.path || '';

        if (dom.ollamaUrl) dom.ollamaUrl.value = config.ollama_url;
        if (dom.batchSize) dom.batchSize.value = config.batch_size;
        if (dom.batchMaxChars) dom.batchMaxChars.value = config.batch_max_chars;
        if (dom.timeoutSec) dom.timeoutSec.value = config.timeout;
        if (dom.packFormat) dom.packFormat.value = config.pack_format ? config.pack_format.toString() : '15';
        if (dom.chkGlossaryPriority) dom.chkGlossaryPriority.checked = config.glossary_priority === 'user';
        if (dom.uiLang) dom.uiLang.value = config.ui_lang;
        if (dom.sourceLang) dom.sourceLang.value = config.source_lang;
        if (dom.targetLang) dom.targetLang.value = config.target_lang || 'zh_tw';
        if (dom.outputDir) dom.outputDir.value = config.output_dir || '';

        if (dom.systemPrompt) dom.systemPrompt.value = config.system_prompt;
        if (dom.userPrompt) dom.userPrompt.value = config.user_prompt;

        if (dom.chkSkipJson) dom.chkSkipJson.checked = config.skip_json || false;
        if (dom.chkSkipJs) dom.chkSkipJs.checked = config.skip_js || false;
        if (dom.chkSkipJar) dom.chkSkipJar.checked = config.skip_jar || false;
        if (dom.chkSkipBook) dom.chkSkipBook.checked = config.skip_book || false;
        if (dom.chkLlmLog) dom.chkLlmLog.checked = config.enable_llm_log || false;
        if (dom.chkDebugTools) dom.chkDebugTools.checked = config.show_debug_tools || false;
        if (config.fast_convert !== undefined) {
            if (dom.chkFastConvert) {
                dom.chkFastConvert.checked = config.fast_convert;
                if (state.toggleLabels?.['chk-fast-convert']) {
                    const stateEl = document.getElementById('label-fast-convert-state');
                    if (stateEl) stateEl.textContent = state.toggleLabels['chk-fast-convert'];
                }
            }
        }
        toggleFastConvertGroup();
        if (dom.excludedPaths && config.excluded_paths) {
            dom.excludedPaths.value = config.excluded_paths.join('\n');
        }

        refreshConfigUiState();
    } catch (e) {
        const mask = state.currentLabels.status_load_config_failed || '❌ 載入配置失敗: {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

export async function saveConfig() {
    try {
        await invoke('save_api_key_cmd', { key: dom.apiKey ? dom.apiKey.value : '' });
        state.currentConfig = await invoke('build_form_config_cmd', {
            base: state.currentConfig,
            input: {
                api_provider: dom.apiProvider ? dom.apiProvider.value : '',
                api_base_url: dom.apiBaseUrl ? dom.apiBaseUrl.value : '',
                ollama_url: dom.ollamaUrl ? dom.ollamaUrl.value : '',
                model: dom.selectedModel ? dom.selectedModel.value : '',
                source_lang: dom.sourceLang ? dom.sourceLang.value : 'en_us',
                target_lang: dom.targetLang ? dom.targetLang.value : 'zh_tw',
                batch_size: dom.batchSize ? dom.batchSize.value : '',
                batch_max_chars: dom.batchMaxChars ? dom.batchMaxChars.value : '',
                timeout: dom.timeoutSec ? dom.timeoutSec.value : '',
                output_dir: dom.outputDir ? dom.outputDir.value : '',
                pack_format: dom.packFormat ? dom.packFormat.value : '',
                user_prompt: dom.userPrompt ? dom.userPrompt.value : '',
                system_prompt: dom.systemPrompt ? dom.systemPrompt.value : '',
                glossary_priority: dom.chkGlossaryPriority && dom.chkGlossaryPriority.checked ? 'user' : 'official',
                skip_json: dom.chkSkipJson ? dom.chkSkipJson.checked : false,
                skip_js: dom.chkSkipJs ? dom.chkSkipJs.checked : false,
                skip_jar: dom.chkSkipJar ? dom.chkSkipJar.checked : false,
                skip_book: dom.chkSkipBook ? dom.chkSkipBook.checked : false,
                enable_llm_log: dom.chkLlmLog ? dom.chkLlmLog.checked : false,
                enable_debug_log: dom.chkDebugLog ? dom.chkDebugLog.checked : false,
                show_debug_tools: dom.chkDebugTools ? dom.chkDebugTools.checked : false,
                ui_lang: dom.uiLang ? dom.uiLang.value : state.currentConfig.ui_lang || 'zh_tw',
                path: dom.inputPath ? dom.inputPath.value : '',
                fast_convert: dom.chkFastConvert ? dom.chkFastConvert.checked : false,
                excluded_paths_text: dom.excludedPaths ? dom.excludedPaths.value : '',
            },
        });
        await invoke('save_config', { config: state.currentConfig });
        refreshConfigUiState();
    } catch (e) {
        const mask = state.currentLabels.status_save_config_failed || '❌ 儲存配置失敗: {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

/** 🟢 根據 dicts/ 目錄動態載入翻譯來源與目標語言 */
export async function loadTranslationLangs() {
    if (!dom.sourceLang || !dom.targetLang) return;

    try {
        const rawLangs = await invoke('get_available_translation_langs');
        const langs = Array.isArray(rawLangs) ? rawLangs : [];

        const populate = (el, currentVal) => {
            el.innerHTML = '';
            langs.forEach((l) => {
                const opt = document.createElement('option');
                opt.value = l;
                const labelKey = `lang_${l}`;
                opt.textContent = state.currentLabels[labelKey] || l;
                el.appendChild(opt);
            });
            if (currentVal) el.value = currentVal;
        };

        populate(dom.sourceLang, state.currentConfig.source_lang);
        populate(dom.targetLang, state.currentConfig.target_lang);

        refreshConfigUiState();
    } catch (e) {
        console.error('無法載入翻譯語言清單:', e);
    }
}

export async function loadModels() {
    if (!dom.apiProvider || !dom.selectedModel) return;
    const provider = dom.apiProvider.value;
    dom.selectedModel.innerHTML = `<option value="">${state.currentLabels.label_loading_models}</option>`;
    try {
        const apiBaseUrlValue = dom.apiBaseUrl ? dom.apiBaseUrl.value : '';
        const models = await invoke('get_models_from_provider', { provider, api_base_url: apiBaseUrlValue });
        dom.selectedModel.innerHTML = `<option value="">${state.currentLabels.prompt_select_model}</option>`;
        if (Array.isArray(models)) {
            models.forEach((m) => {
                const opt = document.createElement('option');
                opt.value = m;
                opt.textContent = m;
                dom.selectedModel.appendChild(opt);
            });
        } else {
            console.warn('models is not an array:', models);
            dom.selectedModel.innerHTML = `<option value="">${state.currentLabels.label_no_models || 'No models'}</option>`;
        }
    } catch (e) {
        console.error('無法載入模型清單:', e);
        const errorLabel = typeof e === 'string' ? state.currentLabels[e] : null;
        if (errorLabel && typeof errorLabel === 'string') {
            dom.selectedModel.innerHTML = `<option value="">${errorLabel}</option>`;
        } else {
            dom.selectedModel.innerHTML = `<option value="">${state.currentLabels.label_no_models || '(無可用模型)'}</option>`;
        }
    }
}

export function toggleOllamaGroup() {
    refreshConfigUiState();
}

/** 當目標語言為中文（zh_cn 或 zh_tw）時顯示快速轉換開關 */
export function toggleFastConvertGroup() {
    refreshConfigUiState();
}

export function toggleApiKeyVisibility() {
    refreshConfigUiState();
}

export function validateCanTranslate() {
    refreshConfigUiState();
}
export async function restoreDefaultConfig() {
    try {
        const defaultConfig = await invoke('get_default_config');

        state.currentConfig.api_provider = defaultConfig.api_provider;
        state.currentConfig.model = defaultConfig.model;
        state.currentConfig.ollama_url = defaultConfig.ollama_url;
        state.currentConfig.api_base_url = defaultConfig.api_base_url;
        state.currentConfig.batch_size = defaultConfig.batch_size;
        state.currentConfig.batch_max_chars = defaultConfig.batch_max_chars;
        state.currentConfig.timeout = defaultConfig.timeout;
        state.currentConfig.glossary_priority = defaultConfig.glossary_priority;
        state.currentConfig.user_prompt = defaultConfig.user_prompt;
        state.currentConfig.fast_convert = defaultConfig.fast_convert || false;

        await invoke('save_api_key_cmd', { key: '' });

        if (dom.apiProvider) dom.apiProvider.value = state.currentConfig.api_provider;
        if (dom.apiKey) dom.apiKey.value = '';
        if (dom.apiBaseUrl) dom.apiBaseUrl.value = state.currentConfig.api_base_url || '';
        if (dom.ollamaUrl) dom.ollamaUrl.value = state.currentConfig.ollama_url;
        if (dom.batchSize) dom.batchSize.value = state.currentConfig.batch_size;
        if (dom.batchMaxChars) dom.batchMaxChars.value = state.currentConfig.batch_max_chars;
        if (dom.timeoutSec) dom.timeoutSec.value = state.currentConfig.timeout;
        if (dom.chkGlossaryPriority) dom.chkGlossaryPriority.checked = state.currentConfig.glossary_priority === 'user';
        if (dom.chkFastConvert) dom.chkFastConvert.checked = state.currentConfig.fast_convert;
        if (dom.systemPrompt) dom.systemPrompt.value = state.currentConfig.system_prompt;
        if (dom.userPrompt) dom.userPrompt.value = state.currentConfig.user_prompt;

        toggleOllamaGroup();
        toggleApiKeyVisibility();
        toggleFastConvertGroup();

        ['chk-glossary-priority', 'chk-fast-convert'].forEach((id) => {
            const el = document.getElementById(id);
            if (el && state.toggleLabels?.[id]) {
                if (id === 'chk-fast-convert') {
                    const stateEl = document.getElementById('label-fast-convert-state');
                    if (stateEl) stateEl.textContent = state.toggleLabels[id];
                } else {
                    const labelEl = document.getElementById(`label-${id.replace('chk-', '')}`);
                    if (labelEl) labelEl.textContent = state.toggleLabels[id];
                }
            }
        });

        await loadModels();

        await invoke('save_config', { config: state.currentConfig });
        appendLog(state.currentLabels.status_config_restored || 'API 設定已恢復預設');
    } catch (e) {
        console.error('恢復 API 預設失敗:', e);
    }
}

export async function restoreDevDefaults() {
    try {
        const defaultConfig = await invoke('get_default_config');

        state.currentConfig.skip_json = defaultConfig.skip_json;
        state.currentConfig.skip_js = defaultConfig.skip_js;
        state.currentConfig.skip_jar = defaultConfig.skip_jar;
        state.currentConfig.skip_book = defaultConfig.skip_book;
        state.currentConfig.enable_llm_log = defaultConfig.enable_llm_log;
        state.currentConfig.enable_debug_log = defaultConfig.enable_debug_log;
        state.currentConfig.system_prompt = defaultConfig.system_prompt;
        state.currentConfig.excluded_paths = defaultConfig.excluded_paths;
        state.currentConfig.show_debug_tools = defaultConfig.show_debug_tools || false;

        if (dom.chkSkipJson) dom.chkSkipJson.checked = state.currentConfig.skip_json;
        if (dom.chkSkipJs) dom.chkSkipJs.checked = state.currentConfig.skip_js;
        if (dom.chkSkipJar) dom.chkSkipJar.checked = state.currentConfig.skip_jar;
        if (dom.chkSkipBook) dom.chkSkipBook.checked = state.currentConfig.skip_book;
        if (dom.chkLlmLog) dom.chkLlmLog.checked = state.currentConfig.enable_llm_log;
        if (dom.chkDebugLog) dom.chkDebugLog.checked = state.currentConfig.enable_debug_log;
        if (dom.chkDebugTools) dom.chkDebugTools.checked = state.currentConfig.show_debug_tools;
        if (dom.systemPrompt) dom.systemPrompt.value = state.currentConfig.system_prompt;
        if (dom.excludedPaths) dom.excludedPaths.value = state.currentConfig.excluded_paths.join('\n');

        [
            'chk-skip-json',
            'chk-skip-js',
            'chk-skip-jar',
            'chk-skip-book',
            'chk-llm-log',
            'chk-debug-log',
            'chk-debug-tools',
        ].forEach((id) => {
            const labelEl = document.getElementById(`label-${id.replace('chk-', '')}`);
            if (labelEl && state.toggleLabels?.[id]) {
                labelEl.textContent = state.toggleLabels[id];
            }
        });

        await invoke('save_config', { config: state.currentConfig });
        appendLog(state.currentLabels.status_dev_restored || '開發人員設定已恢復預設');
    } catch (e) {
        console.error('恢復開發者預設失敗:', e);
    }
}

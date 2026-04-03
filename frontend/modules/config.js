// frontend/modules/config.js
import { state } from './state.js';
import { appendLog } from './utils.js';
import { updateUiLanguage, updateToggleStateLabel } from './i18n.js';

// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);

export async function loadConfig() {
    const apiProvider = document.getElementById('api-provider');
    const apiKey = document.getElementById('api-key');
    const ollamaUrl = document.getElementById('ollama-url');
    const batchSize = document.getElementById('batch-size');
    const batchMaxChars = document.getElementById('batch-max-chars');
    const timeoutSec = document.getElementById('timeout-sec');
    const packFormat = document.getElementById('pack-format');
    const chkGlossaryPriority = document.getElementById('chk-glossary-priority');
    const uiLang = document.getElementById('ui-lang');
    const sourceLang = document.getElementById('source-lang');
    const targetLang = document.getElementById('target-lang');
    const outputDir = document.getElementById('output-dir');
    const systemPrompt = document.getElementById('system-prompt');
    const userPrompt = document.getElementById('user-prompt');
    const chkSkipJson = document.getElementById('chk-skip-json');
    const chkSkipJs = document.getElementById('chk-skip-js');
    const chkSkipJar = document.getElementById('chk-skip-jar');
    const chkSkipBook = document.getElementById('chk-skip-book');
    const chkLlmLog = document.getElementById('chk-llm-log');
    const chkDebugTools = document.getElementById('chk-debug-tools');
    const inputPath = document.getElementById('input-path');

    try {
        const config = await invoke('get_config');

        // [Vite 偵錯優化] 在開發環境下預設開啟開發者模式與偵錯工具
        // 使用 typeof 檢查以確保在各種環境下都不會因為存取此變項報錯
        if (typeof import.meta !== 'undefined' && import.meta.env?.DEV) {
            config.show_developer_mode = true;
            config.show_debug_tools = true;
            console.log('[DEBUG] Vite 環境偵測：已自動開啟開發者模式與偵錯工具');
        }

        state.currentConfig = config;

        if (apiProvider) apiProvider.value = config.api_provider;

        const savedKey = await invoke('get_api_key_cmd');
        if (apiKey) apiKey.value = savedKey || '';

        const apiBaseUrl = document.getElementById('api-base-url');
        if (apiBaseUrl) apiBaseUrl.value = config.api_base_url || '';

        if (inputPath) inputPath.value = config.path || '';

        if (ollamaUrl) ollamaUrl.value = config.ollama_url;
        if (batchSize) batchSize.value = config.batch_size;
        if (batchMaxChars) batchMaxChars.value = config.batch_max_chars;
        if (timeoutSec) timeoutSec.value = config.timeout;
        if (packFormat) packFormat.value = config.pack_format ? config.pack_format.toString() : '15';
        if (chkGlossaryPriority) chkGlossaryPriority.checked = config.glossary_priority === 'user';
        if (uiLang) uiLang.value = config.ui_lang;
        if (sourceLang) sourceLang.value = config.source_lang;
        if (targetLang) targetLang.value = config.target_lang || 'zh_tw';
        if (outputDir) outputDir.value = config.output_dir || '';

        if (systemPrompt) systemPrompt.value = config.system_prompt;
        if (userPrompt) userPrompt.value = config.user_prompt;

        if (chkSkipJson) chkSkipJson.checked = config.skip_json || false;
        if (chkSkipJs) chkSkipJs.checked = config.skip_js || false;
        if (chkSkipJar) chkSkipJar.checked = config.skip_jar || false;
        if (chkSkipBook) chkSkipBook.checked = config.skip_book || false;
        if (chkLlmLog) chkLlmLog.checked = config.enable_llm_log || false;
        if (chkDebugTools) chkDebugTools.checked = config.show_debug_tools || false;
        if (config.fast_convert !== undefined) {
            const chkFast = document.getElementById('chk-fast-convert');
            if (chkFast) {
                chkFast.checked = config.fast_convert;
                updateToggleStateLabel('chk-fast-convert', chkFast.checked);
            }
        }
        toggleFastConvertGroup();
        const excludedPaths = document.getElementById('excluded-paths');
        if (excludedPaths && config.excluded_paths) {
            excludedPaths.value = config.excluded_paths.join('\n');
        }

        toggleOllamaGroup();
        toggleApiKeyVisibility();
        validateCanTranslate();
    } catch (e) {
        const mask = state.currentLabels.status_load_config_failed || '❌ 載入配置失敗: {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

export async function saveConfig() {
    const apiProvider = document.getElementById('api-provider');
    const apiKey = document.getElementById('api-key');
    const selectedModel = document.getElementById('selected-model');
    const ollamaUrl = document.getElementById('ollama-url');
    const batchSize = document.getElementById('batch-size');
    const batchMaxChars = document.getElementById('batch-max-chars');
    const timeoutSec = document.getElementById('timeout-sec');
    const packFormat = document.getElementById('pack-format');
    const chkGlossaryPriority = document.getElementById('chk-glossary-priority');
    const uiLang = document.getElementById('ui-lang');
    const sourceLang = document.getElementById('source-lang');
    const targetLang = document.getElementById('target-lang');
    const outputDir = document.getElementById('output-dir');
    const inputPath = document.getElementById('input-path');
    const systemPrompt = document.getElementById('system-prompt');
    const userPrompt = document.getElementById('user-prompt');
    const chkSkipJson = document.getElementById('chk-skip-json');
    const chkSkipJs = document.getElementById('chk-skip-js');
    const chkSkipJar = document.getElementById('chk-skip-jar');
    const chkSkipBook = document.getElementById('chk-skip-book');
    const chkLlmLog = document.getElementById('chk-llm-log');
    const chkDebugLog = document.getElementById('chk-debug-log');
    const chkDebugTools = document.getElementById('chk-debug-tools');

    try {
        state.currentConfig.api_provider = apiProvider ? apiProvider.value : '';
        // 讀取 API Base URL
        const apiBaseUrl = document.getElementById('api-base-url');
        state.currentConfig.api_base_url = apiBaseUrl ? apiBaseUrl.value : '';

        await invoke('save_api_key_cmd', { key: apiKey ? apiKey.value : '' });
        state.currentConfig.model = selectedModel ? selectedModel.value : '';
        const old = state.currentConfig;
        const parseSafeInt = (v, f) => {
            let p = parseInt(v);
            return isNaN(p) ? f : p;
        };

        state.currentConfig.ollama_url = ollamaUrl ? ollamaUrl.value : '';
        state.currentConfig.batch_size = batchSize ? parseSafeInt(batchSize.value, old.batch_size || 150) : 150;
        state.currentConfig.batch_max_chars = batchMaxChars
            ? parseSafeInt(batchMaxChars.value, old.batch_max_chars || 3500)
            : 3500;
        state.currentConfig.timeout = timeoutSec ? parseSafeInt(timeoutSec.value, old.timeout || 60) : 60;
        if (uiLang) state.currentConfig.ui_lang = uiLang.value;
        if (sourceLang) state.currentConfig.source_lang = sourceLang.value;
        if (targetLang) state.currentConfig.target_lang = targetLang.value;
        state.currentConfig.pack_format = packFormat ? parseSafeInt(packFormat.value, old.pack_format || 15) : 15;
        state.currentConfig.glossary_priority =
            chkGlossaryPriority && chkGlossaryPriority.checked ? 'user' : 'official';
        state.currentConfig.ui_lang = uiLang ? uiLang.value : state.currentConfig.ui_lang || 'zh_tw';
        state.currentConfig.output_dir = outputDir ? outputDir.value : '';
        state.currentConfig.path = inputPath ? inputPath.value : '';

        state.currentConfig.system_prompt = systemPrompt ? systemPrompt.value : '';
        state.currentConfig.user_prompt = userPrompt ? userPrompt.value : '';

        state.currentConfig.skip_json = chkSkipJson ? chkSkipJson.checked : false;
        state.currentConfig.skip_js = chkSkipJs ? chkSkipJs.checked : false;
        state.currentConfig.skip_jar = chkSkipJar ? chkSkipJar.checked : false;
        state.currentConfig.skip_book = chkSkipBook ? chkSkipBook.checked : false;
        state.currentConfig.enable_llm_log = chkLlmLog ? chkLlmLog.checked : false;
        state.currentConfig.enable_debug_log = chkDebugLog ? chkDebugLog.checked : false;
        state.currentConfig.show_debug_tools = chkDebugTools ? chkDebugTools.checked : false;
        const chkFast = document.getElementById('chk-fast-convert');
        if (chkFast) {
            state.currentConfig.fast_convert = chkFast.checked;
        }

        const excludedPaths = document.getElementById('excluded-paths');
        state.currentConfig.excluded_paths = excludedPaths
            ? excludedPaths.value
                  .split('\n')
                  .map((s) => s.trim())
                  .filter((s) => s !== '')
            : [];

        await invoke('save_config', { config: state.currentConfig });
        updateUiLanguage();
    } catch (e) {
        const mask = state.currentLabels.status_save_config_failed || '❌ 儲存配置失敗: {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

/** 🟢 根據 dicts/ 目錄動態載入翻譯來源與目標語言 */
export async function loadTranslationLangs() {
    const sourceLang = document.getElementById('source-lang');
    const targetLang = document.getElementById('target-lang');
    if (!sourceLang || !targetLang) return;

    try {
        const rawLangs = await invoke('get_available_translation_langs');
        const langs = Array.isArray(rawLangs) ? rawLangs : [];

        const populate = (el, currentVal) => {
            el.innerHTML = '';
            langs.forEach((l) => {
                const opt = document.createElement('option');
                opt.value = l;
                // 嘗試從 i18n 標籤中取得顯示名稱 (例如 lang_zh_tw)，無則顯示原始代碼
                const labelKey = `lang_${l}`;
                opt.textContent = state.currentLabels[labelKey] || l;
                el.appendChild(opt);
            });
            if (currentVal) el.value = currentVal;
        };

        populate(sourceLang, state.currentConfig.source_lang);
        populate(targetLang, state.currentConfig.target_lang);

        // 刷新簡繁轉換開關顯示狀態
        toggleFastConvertGroup();
    } catch (e) {
        console.error('無法載入翻譯語言清單:', e);
    }
}

export async function loadModels() {
    const apiProvider = document.getElementById('api-provider');
    const selectedModel = document.getElementById('selected-model');
    if (!apiProvider || !selectedModel) return;
    const provider = apiProvider.value;
    selectedModel.innerHTML = `<option value="">${state.currentLabels.label_loading_models}</option>`;
    try {
        const apiBaseUrl = document.getElementById('api-base-url');
        const apiBaseUrlValue = apiBaseUrl ? apiBaseUrl.value : '';
        const models = await invoke('get_models_from_provider', { provider, api_base_url: apiBaseUrlValue });
        selectedModel.innerHTML = `<option value="">${state.currentLabels.prompt_select_model}</option>`;
        if (Array.isArray(models)) {
            models.forEach((m) => {
                const opt = document.createElement('option');
                opt.value = m;
                opt.textContent = m;
                selectedModel.appendChild(opt);
            });
        } else {
            console.warn('models is not an array:', models);
            selectedModel.innerHTML = `<option value="">${state.currentLabels.label_no_models || 'No models'}</option>`;
        }
    } catch (e) {
        console.error(e);
        selectedModel.innerHTML = `<option value="">${state.currentLabels.label_no_models}</option>`;
    }
}

export function toggleOllamaGroup() {
    const ollamaUrlGroup = document.getElementById('ollama-url-group');
    const apiProvider = document.getElementById('api-provider');
    if (ollamaUrlGroup && apiProvider) {
        ollamaUrlGroup.style.display = apiProvider.value === 'Ollama' ? 'block' : 'none';
    }
}

/** 當目標語言為中文（zh_cn 或 zh_tw）時顯示快速轉換開關 */
export function toggleFastConvertGroup() {
    const group = document.getElementById('group-fast-convert');
    const sourceLang = document.getElementById('source-lang');
    const targetLang = document.getElementById('target-lang');

    if (!group || !sourceLang || !targetLang) {
        return;
    }

    const src = sourceLang.value;
    const tgt = targetLang.value;
    const isTargetChinese = tgt === 'zh_cn' || tgt === 'zh_tw';
    // 來源為中文時（純簡繁互換），目標必須為另一方；非中文來源則只需目標為中文
    const shouldShow = isTargetChinese && src !== tgt;

    group.style.display = shouldShow ? 'block' : 'none';
}

export function toggleApiKeyVisibility() {
    const apiKeyGroup = document.getElementById('api-key-group');
    const apiProvider = document.getElementById('api-provider');
    if (apiKeyGroup && apiProvider) {
        const noKeyProviders = ['Ollama', 'Google Free', '無'];
        apiKeyGroup.style.display = noKeyProviders.includes(apiProvider.value) ? 'none' : 'block';
    }
}

export function validateCanTranslate() {
    const btnTranslate = document.getElementById('btn-translate');
    const selectedModel = document.getElementById('selected-model');
    const apiProvider = document.getElementById('api-provider');
    if (btnTranslate && selectedModel && apiProvider) {
        btnTranslate.disabled =
            !selectedModel.value && apiProvider.value !== 'Google Free' && apiProvider.value !== 'Ollama';
    }
}
export async function restoreDefaultConfig() {
    try {
        const defaultConfig = await invoke('get_default_config');

        // 僅更新與「API 與翻譯設定」相關的欄位
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
        // 移除 excluded_paths 與 system_prompt 的重置（改由 Dev 按鈕負責）

        // 金鑰重置為空 (比照預設值)
        await invoke('save_api_key_cmd', { key: '' });

        // 重新載入 UI
        const apiProvider = document.getElementById('api-provider');
        const apiKey = document.getElementById('api-key');
        const apiBaseUrl = document.getElementById('api-base-url');
        const ollamaUrl = document.getElementById('ollama-url');
        const batchSize = document.getElementById('batch-size');
        const batchMaxChars = document.getElementById('batch-max-chars');
        const timeoutSec = document.getElementById('timeout-sec');
        const chkGlossaryPriority = document.getElementById('chk-glossary-priority');
        const chkFastConvert = document.getElementById('chk-fast-convert');
        const systemPrompt = document.getElementById('system-prompt');
        const userPrompt = document.getElementById('user-prompt');
        // const excludedPaths = document.getElementById('excluded-paths'); // 此處不重置

        if (apiProvider) apiProvider.value = state.currentConfig.api_provider;
        if (apiKey) apiKey.value = '';
        if (apiBaseUrl) apiBaseUrl.value = state.currentConfig.api_base_url || '';
        if (ollamaUrl) ollamaUrl.value = state.currentConfig.ollama_url;
        if (batchSize) batchSize.value = state.currentConfig.batch_size;
        if (batchMaxChars) batchMaxChars.value = state.currentConfig.batch_max_chars;
        if (timeoutSec) timeoutSec.value = state.currentConfig.timeout;
        if (chkGlossaryPriority) chkGlossaryPriority.checked = state.currentConfig.glossary_priority === 'user';
        if (chkFastConvert) chkFastConvert.checked = state.currentConfig.fast_convert;
        if (systemPrompt) systemPrompt.value = state.currentConfig.system_prompt;
        if (userPrompt) userPrompt.value = state.currentConfig.user_prompt;
        // if (excludedPaths) excludedPaths.value = state.currentConfig.excluded_paths.join('\n'); // 此處不重置

        toggleOllamaGroup();
        toggleApiKeyVisibility();
        toggleFastConvertGroup(
            (state.currentConfig.source_lang === 'zh_cn' && state.currentConfig.target_lang === 'zh_tw') ||
                (state.currentConfig.source_lang === 'zh_tw' && state.currentConfig.target_lang === 'zh_cn')
        );

        // 更新 Label 狀態 (切換開關的文字)
        ['chk-glossary-priority', 'chk-fast-convert'].forEach((id) => {
            const el = document.getElementById(id);
            if (el) updateToggleStateLabel(el);
        });

        await loadModels();

        // 自動儲存變更
        await invoke('save_config', { config: state.currentConfig });
        appendLog(state.currentLabels.status_config_restored || 'API 設定已恢復預設');
    } catch (e) {
        console.error('恢復 API 預設失敗:', e);
    }
}

export async function restoreDevDefaults() {
    try {
        const defaultConfig = await invoke('get_default_config');

        // 僅更新與「開發人員選項」相關的欄位
        state.currentConfig.skip_json = defaultConfig.skip_json;
        state.currentConfig.skip_js = defaultConfig.skip_js;
        state.currentConfig.skip_jar = defaultConfig.skip_jar;
        state.currentConfig.skip_book = defaultConfig.skip_book;
        state.currentConfig.enable_llm_log = defaultConfig.enable_llm_log;
        state.currentConfig.enable_debug_log = defaultConfig.enable_debug_log;
        state.currentConfig.system_prompt = defaultConfig.system_prompt;
        state.currentConfig.excluded_paths = defaultConfig.excluded_paths;
        state.currentConfig.show_debug_tools = defaultConfig.show_debug_tools || false;

        // 重新載入 UI
        const chkSkipJson = document.getElementById('chk-skip-json');
        const chkSkipJs = document.getElementById('chk-skip-js');
        const chkSkipJar = document.getElementById('chk-skip-jar');
        const chkSkipBook = document.getElementById('chk-skip-book');
        const chkLlmLog = document.getElementById('chk-llm-log');
        const chkDebugLog = document.getElementById('chk-debug-log');
        const chkDebugTools = document.getElementById('chk-debug-tools');
        const systemPrompt = document.getElementById('system-prompt');
        const excludedPaths = document.getElementById('excluded-paths');

        if (chkSkipJson) chkSkipJson.checked = state.currentConfig.skip_json;
        if (chkSkipJs) chkSkipJs.checked = state.currentConfig.skip_js;
        if (chkSkipJar) chkSkipJar.checked = state.currentConfig.skip_jar;
        if (chkSkipBook) chkSkipBook.checked = state.currentConfig.skip_book;
        if (chkLlmLog) chkLlmLog.checked = state.currentConfig.enable_llm_log;
        if (chkDebugLog) chkDebugLog.checked = state.currentConfig.enable_debug_log;
        if (chkDebugTools) chkDebugTools.checked = state.currentConfig.show_debug_tools;
        if (systemPrompt) systemPrompt.value = state.currentConfig.system_prompt;
        if (excludedPaths) excludedPaths.value = state.currentConfig.excluded_paths.join('\n');

        // 更新 Label 狀態 (切換開關的文字)
        [
            'chk-skip-json',
            'chk-skip-js',
            'chk-skip-jar',
            'chk-skip-book',
            'chk-llm-log',
            'chk-debug-log',
            'chk-debug-tools',
        ].forEach((id) => {
            const el = document.getElementById(id);
            if (el) updateToggleStateLabel(el);
        });

        // 自動儲存變更
        await invoke('save_config', { config: state.currentConfig });
        appendLog(state.currentLabels.status_dev_restored || '開發人員設定已恢復預設');
    } catch (e) {
        console.error('恢復開發者預設失敗:', e);
    }
}

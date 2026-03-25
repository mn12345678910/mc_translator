// frontend/modules/config.js
import { state } from './state.js';
import { appendLog } from './utils.js';
import { updateUiLanguage } from './i18n.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };

export async function loadConfig() {
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
    const systemPrompt = document.getElementById('system-prompt');
    const userPrompt = document.getElementById('user-prompt');
    const chkSkipJson = document.getElementById('chk-skip-json');
    const chkSkipJs = document.getElementById('chk-skip-js');
    const chkSkipJar = document.getElementById('chk-skip-jar');
    const chkSkipBook = document.getElementById('chk-skip-book');
    const chkLlmLog = document.getElementById('chk-llm-log');
    const inputPath = document.getElementById('input-path');

    try {
        const config = await invoke('get_config');
        state.currentConfig = config;

        if (apiProvider) apiProvider.value = config.api_provider;

        const savedKey = await invoke('get_api_key_cmd');
        if (apiKey) apiKey.value = savedKey || '';

        if (inputPath) inputPath.value = config.path || '';

        if (ollamaUrl) ollamaUrl.value = config.ollama_url;
        if (batchSize) batchSize.value = config.batch_size;
        if (batchMaxChars) batchMaxChars.value = config.batch_max_chars;
        if (timeoutSec) timeoutSec.value = config.timeout;
        if (packFormat) packFormat.value = config.pack_format ? config.pack_format.toString() : '15';
        if (chkGlossaryPriority) chkGlossaryPriority.checked = config.glossary_priority === 'user';
        if (uiLang) uiLang.value = config.ui_lang;
        if (sourceLang) sourceLang.value = config.source_lang;
        if (targetLang) targetLang.value = config.target_lang;
        if (outputDir) outputDir.value = config.output_dir;

        if (systemPrompt) systemPrompt.value = config.system_prompt;
        if (userPrompt) userPrompt.value = config.user_prompt;

        if (chkSkipJson) chkSkipJson.checked = config.skip_json || false;
        if (chkSkipJs) chkSkipJs.checked = config.skip_js || false;
        if (chkSkipJar) chkSkipJar.checked = config.skip_jar || false;
        if (chkSkipBook) chkSkipBook.checked = config.skip_book || false;
        if (chkLlmLog) chkLlmLog.checked = config.enable_llm_log || false;

        toggleOllamaGroup();
        await updateUiLanguage();
        await loadModels();
        if (config.model && selectedModel) {
            selectedModel.value = config.model;
        }
        toggleApiKeyVisibility();
        validateCanTranslate();
    } catch (e) {
        const mask = state.currentLabels.status_load_config_failed;
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

    try {
        state.currentConfig.api_provider = apiProvider ? apiProvider.value : '';
        await invoke('save_api_key_cmd', { key: apiKey ? apiKey.value : '' });
        state.currentConfig.model = selectedModel ? selectedModel.value : '';
        const old = state.currentConfig;
        const parseSafeInt = (v, f) => { let p = parseInt(v); return isNaN(p) ? f : p; };

        state.currentConfig.ollama_url = ollamaUrl ? ollamaUrl.value : '';
        state.currentConfig.batch_size = batchSize ? parseSafeInt(batchSize.value, old.batch_size || 150) : 150;
        state.currentConfig.batch_max_chars = batchMaxChars ? parseSafeInt(batchMaxChars.value, old.batch_max_chars || 3500) : 3500;
        state.currentConfig.timeout = timeoutSec ? parseSafeInt(timeoutSec.value, old.timeout || 60) : 60;
        if (uiLang) state.currentConfig.ui_lang = uiLang.value;
        if (sourceLang) state.currentConfig.source_lang = sourceLang.value;
        if (targetLang) state.currentConfig.target_lang = targetLang.value;
        state.currentConfig.pack_format = packFormat ? parseSafeInt(packFormat.value, old.pack_format || 15) : 15;
        state.currentConfig.glossary_priority = chkGlossaryPriority && chkGlossaryPriority.checked ? 'user' : 'official';
        state.currentConfig.ui_lang = uiLang ? uiLang.value : (state.currentConfig.ui_lang || 'zh_tw');
        state.currentConfig.output_dir = outputDir ? outputDir.value : '';
        state.currentConfig.path = inputPath ? inputPath.value : '';

        state.currentConfig.system_prompt = systemPrompt ? systemPrompt.value : '';
        state.currentConfig.user_prompt = userPrompt ? userPrompt.value : '';

        state.currentConfig.skip_json = chkSkipJson ? chkSkipJson.checked : false;
        state.currentConfig.skip_js = chkSkipJs ? chkSkipJs.checked : false;
        state.currentConfig.skip_jar = chkSkipJar ? chkSkipJar.checked : false;
        state.currentConfig.skip_book = chkSkipBook ? chkSkipBook.checked : false;
        state.currentConfig.enable_llm_log = chkLlmLog ? chkLlmLog.checked : false;

        await invoke('save_config', { config: state.currentConfig });
        updateUiLanguage();
    } catch (e) {
        const mask = state.currentLabels.status_save_config_failed;
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

export async function loadModels() {
    const apiProvider = document.getElementById('api-provider');
    const selectedModel = document.getElementById('selected-model');
    if (!apiProvider || !selectedModel) return;
    const provider = apiProvider.value;
    selectedModel.innerHTML = `<option value="">${state.currentLabels.label_loading_models}</option>`;
    try {
        const models = await invoke('get_models_from_provider', { provider });
        selectedModel.innerHTML = `<option value="">${state.currentLabels.prompt_select_model}</option>`;
        models.forEach((m) => {
            const opt = document.createElement('option');
            opt.value = m;
            opt.textContent = m;
            selectedModel.appendChild(opt);
        });
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
        state.currentConfig.system_prompt = defaultConfig.system_prompt;
        state.currentConfig.user_prompt = defaultConfig.user_prompt;

        // 金鑰重置為空 (比照預設值)
        await invoke('save_api_key_cmd', { key: '' });
        
        // 重新載入 UI
        const apiProvider = document.getElementById('api-provider');
        const apiKey = document.getElementById('api-key');
        const ollamaUrl = document.getElementById('ollama-url');
        const batchSize = document.getElementById('batch-size');
        const batchMaxChars = document.getElementById('batch-max-chars');
        const timeoutSec = document.getElementById('timeout-sec');
        const chkGlossaryPriority = document.getElementById('chk-glossary-priority');
        const systemPrompt = document.getElementById('system-prompt');
        const userPrompt = document.getElementById('user-prompt');

        if (apiProvider) apiProvider.value = state.currentConfig.api_provider;
        if (apiKey) apiKey.value = '';
        if (ollamaUrl) ollamaUrl.value = state.currentConfig.ollama_url;
        if (batchSize) batchSize.value = state.currentConfig.batch_size;
        if (batchMaxChars) batchMaxChars.value = state.currentConfig.batch_max_chars;
        if (timeoutSec) timeoutSec.value = state.currentConfig.timeout;
        if (chkGlossaryPriority) chkGlossaryPriority.checked = state.currentConfig.glossary_priority === 'user';
        if (systemPrompt) systemPrompt.value = state.currentConfig.system_prompt;
        if (userPrompt) userPrompt.value = state.currentConfig.user_prompt;

        toggleOllamaGroup();
        toggleApiKeyVisibility();
        await loadModels();
        
        // 自動儲存變更
        await invoke('save_config', { config: state.currentConfig });
        appendLog(state.currentLabels.status_config_restored || 'API 設定已恢復預設');
    } catch (e) {
        console.error('恢復 API 預設失敗:', e);
    }
}

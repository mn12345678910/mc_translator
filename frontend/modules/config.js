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
        if (batchSize) batchSize.value = config.batch_size || 150;
        if (batchMaxChars) batchMaxChars.value = config.batch_max_chars || 3500;
        if (timeoutSec) timeoutSec.value = config.timeout || 60;
        if (packFormat) packFormat.value = config.pack_format ? config.pack_format.toString() : '15';
        if (chkGlossaryPriority) chkGlossaryPriority.checked = config.glossary_priority === 'user';
        if (uiLang) uiLang.value = config.ui_lang;
        if (sourceLang) sourceLang.value = config.source_lang || 'en_us';
        if (targetLang) targetLang.value = config.target_lang || 'zh_tw';
        if (outputDir) outputDir.value = config.output_dir || '';

        if (systemPrompt) systemPrompt.value = config.system_prompt || '';
        if (userPrompt) userPrompt.value = config.user_prompt || '';

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
        state.currentConfig.ollama_url = ollamaUrl ? ollamaUrl.value : 'http://localhost:11434';
        state.currentConfig.batch_size = batchSize ? parseInt(batchSize.value) : 150;
        state.currentConfig.batch_max_chars = batchMaxChars ? parseInt(batchMaxChars.value) : 3500;
        state.currentConfig.timeout = timeoutSec ? parseInt(timeoutSec.value) : 60;
        if (uiLang) state.currentConfig.ui_lang = uiLang.value;
        if (sourceLang) state.currentConfig.source_lang = sourceLang.value;
        if (targetLang) state.currentConfig.target_lang = targetLang.value;
        state.currentConfig.pack_format = packFormat ? parseInt(packFormat.value) : 15;
        state.currentConfig.glossary_priority = chkGlossaryPriority && chkGlossaryPriority.checked ? 'user' : 'official';
        state.currentConfig.ui_lang = uiLang ? uiLang.value : 'zh_tw';
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
        appendLog(state.currentLabels.status_save_config_success);
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

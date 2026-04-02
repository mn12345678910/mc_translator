// frontend/modules/i18n.js
import { state } from './state.js';
import { loadTranslationLangs } from './config.js';
// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);

export async function loadUiLangs() {
    const uiLang = document.getElementById('ui-lang');
    if (!uiLang) return;
    try {
        const rawLangs = await invoke('get_available_langs');
        const langs = Array.isArray(rawLangs) ? rawLangs : [];
        uiLang.innerHTML = '';
        const allLangs = Array.from(new Set([...langs, 'zh_tw', 'zh_cn', 'en_us', 'ja_jp']));
        allLangs.forEach((l) => {
            const opt = document.createElement('option');
            opt.value = l;
            opt.textContent =
                l === 'zh_tw'
                    ? state.currentLabels.lang_zh_tw
                    : l === 'en_us'
                      ? state.currentLabels.lang_en_us
                      : l === 'zh_cn'
                        ? state.currentLabels.lang_zh_cn
                        : l === 'ja_jp'
                          ? state.currentLabels.lang_ja_jp
                          : l;
            uiLang.appendChild(opt);
        });

        if (uiLang && state.currentConfig && state.currentConfig.ui_lang) {
            uiLang.value = state.currentConfig.ui_lang;
        }
    } catch (e) {
        console.error('無法載入語言清單', e);
    }
}

export async function updateUiLanguage() {
    const uiLang = document.getElementById('ui-lang');
    const btnNavApi = document.getElementById('btn-nav-api');
    const btnNavDict = document.getElementById('btn-nav-dict');
    const btnNavPalette = document.getElementById('btn-nav-palette');
    const btnNavTheme = document.getElementById('btn-nav-theme');
    const btnNavDev = document.getElementById('btn-nav-dev');

    try {
        const labels = await invoke('get_i18n_labels', { lang: uiLang ? uiLang.value : undefined });
        if (!labels) return;
        const oldLabels = { ...state.currentLabels };
        state.currentLabels = { ...labels };

        // 先更新 UI 語言下拉顯示文字（需要 labels 就緒）
        await loadUiLangs();

        if (uiLang && uiLang.value) {
            document.documentElement.lang = uiLang.value.replace('_', '-');
        }

        // 🟢 更新 <title> 標籤
        if (labels.page_title) document.title = labels.page_title;

        const titleNode = document.querySelector('h1 span') || document.querySelector('h1');
        if (titleNode && labels.app_title) titleNode.textContent = labels.app_title;
        // 🟡 舊有 Mapping 映射已移除，全面採用屬性驅動

        // 🟢 1. 執行屬性驅動的通用映射 (textContent)
        document.querySelectorAll('[data-i18n]').forEach((el) => {
            const key = el.getAttribute('data-i18n');
            if (labels[key]) el.textContent = labels[key];
        });

        // 🟢 2. 執行標籤映射 (data-i18n-label for optgroup)
        document.querySelectorAll('[data-i18n-label]').forEach((el) => {
            const key = el.getAttribute('data-i18n-label');
            if (labels[key]) el.label = labels[key];
        });

        // 🟢 3. 執行 Placeholder 映射 (data-i18n-placeholder)
        document.querySelectorAll('[data-i18n-placeholder]').forEach((el) => {
            const key = el.getAttribute('data-i18n-placeholder');
            if (labels[key]) el.placeholder = labels[key];
        });

        // 🟢 4. 執行 Title 懸停提示映射 (data-i18n-title)
        document.querySelectorAll('[data-i18n-title]').forEach((el) => {
            const key = el.getAttribute('data-i18n-title');
            if (labels[key]) el.title = labels[key];
        });

        // 移除原有的手動 optionMapping 循環

        // 已在函式開頭宣告過，直接使用

        if (labels.btn_nav_settings && btnNavApi) btnNavApi.title = labels.btn_nav_settings;
        if (labels.btn_nav_dict && btnNavDict) btnNavDict.title = labels.btn_nav_dict;
        if (labels.btn_nav_palette && btnNavPalette) btnNavPalette.title = labels.btn_nav_palette;
        if (labels.btn_nav_theme && btnNavTheme) btnNavTheme.title = labels.btn_nav_theme;
        if (labels.btn_nav_dev && btnNavDev) btnNavDev.title = labels.btn_nav_dev;

        // 🟢 彈窗標題 (Header) 翻譯
        const dictHeader = document.getElementById('header-dict-mgr');
        if (dictHeader && labels.header_dict_mgr) {
            dictHeader.textContent = labels.header_dict_mgr;
        }

        // 🟢 額外懸停提示 (Tooltip) 翻譯
        const btnOpenJson = document.getElementById('btn-dict-open-json');
        if (btnOpenJson && labels.btn_dict_open_json) btnOpenJson.title = labels.btn_dict_open_json;

        const chkGlossary = document.getElementById('chk-glossary-priority');
        if (chkGlossary && chkGlossary.parentElement && labels.glossary_priority_hover) {
            chkGlossary.parentElement.title = labels.glossary_priority_hover;
        }

        document.querySelectorAll('input[placeholder], textarea[placeholder]').forEach((el) => {
            const id = el.id;
            if (!id) return;
            const underscored = id.replace(/-/g, '_');
            const key = `placeholder_${underscored}`;
            if (id === 'dict-search' && labels.placeholder_search_terms)
                el.placeholder = labels.placeholder_search_terms;
            else if (id === 'dict-input-key' && labels.placeholder_dict_key)
                el.placeholder = labels.placeholder_dict_key;
            else if (id === 'dict-input-value' && labels.placeholder_dict_value)
                el.placeholder = labels.placeholder_dict_value;
            else if (id === 'input-path' && labels.placeholder_input_path)
                el.placeholder = labels.placeholder_input_path;
            else if (labels[key]) el.placeholder = labels[key];
        });

        // NOTE: UI 語系切換不再影響使用者/系統提示

        const selectedModel = document.getElementById('selected-model');
        if (selectedModel && selectedModel.options.length > 0) {
            const firstOpt = selectedModel.options[0];
            if (firstOpt.value === '') {
                const oldSelect = oldLabels.prompt_select_model;
                const oldLoading = oldLabels.label_loading_models;
                const oldNoModels = oldLabels.label_no_models;
                if (
                    firstOpt.textContent === oldSelect ||
                    firstOpt.textContent === oldLoading ||
                    firstOpt.textContent === oldNoModels
                ) {
                    firstOpt.textContent = labels.prompt_select_model;
                }
            }
        }

        // 🟢 根據開關狀態刷新 Label 文字 (切換語系時一併觸發)
        const allSwitches = [
            'chk-glossary-priority',
            'chk-skip-json',
            'chk-skip-js',
            'chk-skip-jar',
            'chk-skip-book',
            'chk-llm-log',
            'chk-debug-log',
            'chk-debug-tools',
            'chk-fast-convert',
        ];
        allSwitches.forEach((id) => {
            const toggleEl = document.getElementById(id);
            if (toggleEl) updateToggleStateLabel(id, toggleEl.checked);
        });

        if (window.__TAURI__ && window.__TAURI__.event) {
            window.__TAURI__.event.emit('ui-lang-changed', uiLang ? uiLang.value : undefined);
        }

        // 🟢 重新渲染翻譯語言下拉標籤 (由於語系變換需要顯示各語系的翻譯名稱)
        await loadTranslationLangs();
    } catch (err) {
        console.error('Failed to update UI language:', err);
    }
}

// 🟢 依照開關撥動狀態，點按時動態且立即更新對應文字 Label
export function updateToggleStateLabel(id, checked) {
    const labels = (typeof state !== 'undefined' && state.currentLabels) || {};

    // 🔴 [FIX] 針對 chk-fast-convert 特殊處理，因其標籤 ID 是 label-fast-convert-state
    if (id === 'chk-fast-convert') {
        const stateEl = document.getElementById('label-fast-convert-state');
        if (stateEl) {
            stateEl.textContent = checked ? labels.label_fast_convert_on : labels.label_fast_convert_off;
        }
        return;
    }

    const labelEl = document.getElementById(`label-${id.replace('chk-', '')}`);
    if (!labelEl) return;

    if (id === 'chk-glossary-priority') {
        labelEl.textContent = checked ? labels.glossary_priority_user : labels.glossary_priority_official;
    } else if (id === 'chk-llm-log') {
        labelEl.textContent = checked ? labels.label_enable_log : labels.label_disable_log;
    } else if (id === 'chk-skip-json') {
        labelEl.textContent = checked ? labels.label_skip_json : labels.label_no_skip_json;
    } else if (id === 'chk-skip-js') {
        labelEl.textContent = checked ? labels.label_skip_js : labels.label_no_skip_js;
    } else if (id === 'chk-skip-jar') {
        labelEl.textContent = checked ? labels.label_skip_jar : labels.label_no_skip_jar;
    } else if (id === 'chk-skip-book') {
        labelEl.textContent = checked ? labels.label_skip_book : labels.label_no_skip_book;
    } else if (id === 'chk-debug-log') {
        labelEl.textContent = checked ? labels.label_enable_debug_log : labels.label_disable_debug_log;
    } else if (id === 'chk-debug-tools') {
        labelEl.textContent = checked ? labels.label_hide_debug_tools : labels.label_show_debug_tools;
    }
}

// frontend/modules/i18n.js
import { state } from './state.js';
import { loadTranslationLangs } from './config.js';
import { dom } from './dom.js';
// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);

export async function loadUiLangs() {
    if (!dom.uiLang) return;
    try {
        const rawLangs = await invoke('get_available_langs');
        const langs = Array.isArray(rawLangs) ? rawLangs : [];
        dom.uiLang.innerHTML = '';
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
            dom.uiLang.appendChild(opt);
        });

        if (dom.uiLang && state.currentConfig && state.currentConfig.ui_lang) {
            dom.uiLang.value = state.currentConfig.ui_lang;
        }
    } catch (e) {
        console.error('無法載入語言清單', e);
    }
}

export async function updateUiLanguage() {
    try {
        const labels = await invoke('get_i18n_labels', { lang: dom.uiLang ? dom.uiLang.value : undefined });
        if (!labels) return;
        const oldLabels = { ...state.currentLabels };
        state.currentLabels = { ...labels };

        // 先更新 UI 語言下拉顯示文字（需要 labels 就緒）
        await loadUiLangs();

        if (dom.uiLang && dom.uiLang.value) {
            document.documentElement.lang = dom.uiLang.value.replace('_', '-');
        }

        // 🟢 更新 <title> 標籤
        if (labels.page_title) document.title = labels.page_title;

        const titleNode = document.querySelector('h1 span') || document.querySelector('h1');
        if (titleNode && labels.app_title) titleNode.textContent = labels.app_title;

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

        if (labels.btn_nav_settings && dom.btnNavApi) dom.btnNavApi.title = labels.btn_nav_settings;
        if (labels.btn_nav_dict && dom.btnNavDict) dom.btnNavDict.title = labels.btn_nav_dict;
        if (labels.btn_nav_palette && dom.btnNavPalette) dom.btnNavPalette.title = labels.btn_nav_palette;
        if (labels.btn_nav_theme && dom.btnNavTheme) dom.btnNavTheme.title = labels.btn_nav_theme;
        if (labels.btn_nav_dev && dom.btnNavDev) dom.btnNavDev.title = labels.btn_nav_dev;

        // 🟢 彈窗標題 (Header) 翻譯
        if (dom.headerDictMgr && labels.header_dict_mgr) {
            dom.headerDictMgr.textContent = labels.header_dict_mgr;
        }

        // 🟢 額外懸停提示 (Tooltip) 翻譯
        if (dom.chkGlossaryPriority && dom.chkGlossaryPriority.parentElement && labels.glossary_priority_hover) {
            dom.chkGlossaryPriority.parentElement.title = labels.glossary_priority_hover;
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

        if (dom.selectedModel && dom.selectedModel.options.length > 0) {
            const firstOpt = dom.selectedModel.options[0];
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

        // 🟢 直接使用本地 toggleLabels 更新 DOM（從 initState 載入）
        const switchIds = [
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
        switchIds.forEach((id) => {
            const labelEl = document.getElementById(`label-${id.replace('chk-', '')}`);
            if (labelEl && state.toggleLabels?.[id]) {
                if (id === 'chk-fast-convert') {
                    const stateEl = document.getElementById('label-fast-convert-state');
                    if (stateEl) stateEl.textContent = state.toggleLabels[id];
                } else {
                    labelEl.textContent = state.toggleLabels[id];
                }
            }
        });

        if (window.__TAURI__ && window.__TAURI__.event) {
            window.__TAURI__.event.emit('ui-lang-changed', dom.uiLang ? dom.uiLang.value : undefined);
        }

        // 🟢 重新渲染翻譯語言下拉標籤
        await loadTranslationLangs();
    } catch (err) {
        console.error('Failed to update UI language:', err);
    }
}

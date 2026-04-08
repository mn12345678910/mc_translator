// frontend/main.js
import { state } from './modules/state.js';
import { debounce, appendLog } from './modules/utils.js';
import { loadUiLangs, updateUiLanguage, updateToggleStateLabel } from './modules/i18n.js';
import {
    loadConfig,
    loadTranslationLangs,
    loadModels,
    saveConfig,
    restoreDefaultConfig,
    restoreDevDefaults,
    toggleOllamaGroup,
    toggleApiKeyVisibility,
    toggleFastConvertGroup,
    validateCanTranslate,
} from './modules/config.js';
import { loadStyle, saveStyle, restoreDefaultStyle, applyColors, updatePaletteValue } from './modules/style.js';
import { initDictionary, loadDictionary } from './modules/dictionary.js';
import { initTranslation } from './modules/translation.js';
import { VirtualLogViewer } from './modules/virtual_log.js';
import { dom } from './modules/dom.js';

// 動態取得 invoke，防止在 Mock 載入前就被靜態截流 (支援 Vite 瀏覽器偵錯)
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);

document.addEventListener('DOMContentLoaded', async () => {
    const applyCssVars = (cssVars) => {
        if (!cssVars || typeof cssVars !== 'object') return;
        const root = document.documentElement;
        Object.entries(cssVars).forEach(([key, value]) => {
            root.style.setProperty(key, value);
        });
    };

    // Rust-first init snapshot: keep frontend as thin renderer.
    try {
        const initState = await invoke('get_gui_init_state');
        if (initState && typeof initState === 'object') {
            if (initState.config) state.currentConfig = initState.config;
            if (initState.style) state.currentStyle = initState.style;
            if (initState.labels) state.currentLabels = initState.labels;
            applyCssVars(initState.css_vars);
        }
    } catch (e) {
        console.warn('Failed to load Rust GUI init state, fallback to legacy startup:', e);
    }

    // [NEW] 1. 優先在開發模式下載入 Mock 工具，確保後續 invoke 正常
    if (typeof import.meta !== 'undefined' && import.meta.env?.DEV) {
        if (window.__TAURI__) {
            try {
                await invoke('setup_dev_mock');
            } catch (e) {
                console.warn('Failed to setup Rust dev mock:', e);
            }
        } else {
            try {
                const { initMockTools } = await import('./modules/mock.js');
                await initMockTools();
                console.log('[DEBUG] Browser mock tools initialized before config load.');
            } catch (e) {
                console.error('Failed to pre-load browser mock tools:', e);
            }
        }
    }

    // 2. 初始化資料載入 (嚴格確保載入順序)
    await loadConfig(); // 載入設定 (決定 UI 語言與基礎路徑)
    await loadUiLangs(); // 載入介面語言選單
    await updateUiLanguage(); // 根據設定更新介面標籤 (同步 labels)
    await loadTranslationLangs(); // 根據 labels 填充翻譯選單
    await loadModels(); // 根據 labels 填充模型選單
    if (state.currentConfig.model) {
        if (dom.selectedModel) dom.selectedModel.value = state.currentConfig.model;
    }
    await loadStyle(); // 載入視覺樣式
    await loadDictionary(); // 載入辭典快取

    if (window.__TAURI__) {
        invoke('show_window');
    }

    // 2. 初始化子模組事件綁定
    window.__logViewer = new VirtualLogViewer('log-output', {
        onUpdate: (stats) => {
            if (dom.debugRenderedCount) dom.debugRenderedCount.textContent = stats.rendered;
            if (dom.debugScrollLocked) dom.debugScrollLocked.textContent = stats.isLocked ? 'True' : 'False';
            if (dom.debugTotalLogs) dom.debugTotalLogs.textContent = stats.total.toLocaleString();
            if (dom.debugMemoryEst) {
                const estMB = Math.round((stats.total * 300) / (1024 * 1024));
                dom.debugMemoryEst.textContent = `~${estMB} MB`;
            }
        },
    });

    initDictionary();
    initTranslation();

    // 3. 基礎按鈕點擊綁定 ( browse 等 )
    async function browsePath(type, targetEl) {
        try {
            const path = await invoke('open_path_dialog', { diagType: type });
            if (path && targetEl) targetEl.value = path;
        } catch (e) {
            const mask = state.currentLabels.status_browse_path_failed;
            appendLog(mask.replace('{}', state.currentLabels[e] || e));
        }
    }

    if (dom.btnBrowseFile) dom.btnBrowseFile.addEventListener('click', () => browsePath('file', dom.inputPath));
    if (dom.btnBrowseDir) dom.btnBrowseDir.addEventListener('click', () => browsePath('dir', dom.inputPath));
    if (dom.btnBrowseOutput) dom.btnBrowseOutput.addEventListener('click', () => browsePath('dir', dom.outputDir));

    if (dom.btnBrowseOutputOpen) {
        dom.btnBrowseOutputOpen.addEventListener('click', async () => {
            const target = dom.outputDir ? dom.outputDir.value.trim() : '';
            try {
                await invoke('open_folder', { path: target });
            } catch (e) {
                console.error(e);
            }
        });
    }

    // 4. 下拉選單與防抖自動儲存
    if (dom.apiProvider) {
        dom.apiProvider.addEventListener('change', async () => {
            toggleOllamaGroup();
            toggleApiKeyVisibility();
            const loadModelsModule = await import('./modules/config.js').then((m) => m.loadModels);
            await loadModelsModule();
            validateCanTranslate();
            debouncedSaveConfig();
        });
    }

    const debouncedSaveConfig = debounce(async () => {
        await saveConfig();
    }, 500);
    const debouncedSaveStyle = debounce(async () => {
        await saveStyle();
    }, 500);

    const configInputs = [
        'api-key',
        'api-base-url',
        'ollama-url',
        'batch-size',
        'batch-max-chars',
        'timeout-sec',
        'user-prompt',
        'system-prompt',
        'excluded-paths',
    ];
    configInputs.forEach((id) => {
        const inputEl = document.getElementById(id);
        if (inputEl) {
            inputEl.addEventListener('input', debouncedSaveConfig);
            if (id === 'api-key') {
                inputEl.addEventListener('blur', async () => {
                    await saveConfig();
                    const loadModelsModule = await import('./modules/config.js').then((m) => m.loadModels);
                    await loadModelsModule();
                });
            } else {
                inputEl.addEventListener('blur', () => saveConfig()); // 立即儲存
            }
        }
    });

    const styleInputs = ['font-size', 'btn-rounding-value', 'pulse-speed'];
    styleInputs.forEach((id) => {
        const inputEl = document.getElementById(id);
        if (inputEl) {
            inputEl.addEventListener('input', debouncedSaveStyle);
            inputEl.addEventListener('blur', () => saveStyle()); // 立即儲存
        }
    });

    const configSelects = [
        'pack-format',
        'chk-glossary-priority',
        'chk-skip-json',
        'chk-skip-js',
        'chk-skip-jar',
        'chk-skip-book',
        'chk-llm-log',
        'chk-debug-log',
        'chk-debug-tools',
        'chk-fast-convert',
        'source-lang',
        'target-lang',
        'selected-model',
    ];
    configSelects.forEach((id) => {
        const selectEl = document.getElementById(id);
        if (selectEl) {
            selectEl.addEventListener('change', async () => {
                if (id.startsWith('chk-')) {
                    updateToggleStateLabel(id, selectEl.checked);
                }

                // 處理簡繁快速轉換開關顯示
                if (id === 'source-lang' || id === 'target-lang') {
                    toggleFastConvertGroup();
                }

                // 目標語言變更時載入預設 Prompt
                if (id === 'target-lang') {
                    try {
                        const prompts = await invoke('derive_default_prompts', { lang: selectEl.value });
                        if (dom.userPrompt && prompts.default_user_prompt) {
                            dom.userPrompt.value = prompts.default_user_prompt;
                        }
                        if (dom.systemPrompt && prompts.default_system_prompt) {
                            dom.systemPrompt.value = prompts.default_system_prompt;
                        }
                    } catch (e) {
                        console.error('載入預設 Prompts 失敗:', e);
                    }
                }

                debouncedSaveConfig();
            });
        }
    });

    const styleSelects = ['chk-btn-rounding', 'chk-pulse', 'progress-style'];
    styleSelects.forEach((id) => {
        const selectEl = document.getElementById(id);
        if (selectEl) selectEl.addEventListener('change', debouncedSaveStyle);
    });

    if (dom.uiLang) {
        dom.uiLang.addEventListener('change', async () => {
            state.currentConfig.ui_lang = dom.uiLang.value;
            document.documentElement.lang = dom.uiLang.value.replace('_', '-');
            await invoke('save_config', { config: state.currentConfig });
            await updateUiLanguage();
        });
    }

    // 移除舊的重複監聽器區塊

    // 5. 導覽面板控制
    const panelApi = document.querySelector('.api-settings');
    const panelDev = document.querySelector('.developer-settings');
    const panelTheme = document.querySelector('.theme-settings');

    function updatePanelVisibility(panelState) {
        const showApi = panelState ? panelState.show_api_settings : !!state.currentConfig.show_api_settings;
        const showDev = panelState ? panelState.show_developer_mode : !!state.currentConfig.show_developer_mode;
        const showPalette = panelState ? panelState.show_palette_settings : !!state.currentStyle.show_palette_settings;

        if (panelApi) panelApi.classList.toggle('expanded', showApi);
        if (panelDev) panelDev.classList.toggle('expanded', showDev);
        if (panelTheme) panelTheme.classList.toggle('expanded', showPalette);

        // 控制偵錯工具開關的顯示 (僅在開發者模式展開時才顯示開關本身)
        const groupDebugTools = document.getElementById('group-debug-tools');
        if (groupDebugTools) {
            groupDebugTools.style.display = showDev ? 'flex' : 'none';
        }
    }

    async function applyPanelAction(action) {
        const panelState = await invoke('derive_panel_state_cmd', {
            action,
            current: {
                show_api_settings: !!state.currentConfig.show_api_settings,
                show_developer_mode: !!state.currentConfig.show_developer_mode,
                show_palette_settings: !!state.currentStyle.show_palette_settings,
            },
        });
        state.currentConfig.show_api_settings = !!panelState.show_api_settings;
        state.currentConfig.show_developer_mode = !!panelState.show_developer_mode;
        state.currentStyle.show_palette_settings = !!panelState.show_palette_settings;
        updatePanelVisibility(panelState);
        await invoke('save_config', { config: state.currentConfig });
    }

    if (dom.btnNavApi) {
        dom.btnNavApi.addEventListener('click', async () => {
            await applyPanelAction('toggle_api');
        });
    }
    if (dom.btnNavDev) {
        dom.btnNavDev.addEventListener('click', async () => {
            await applyPanelAction('toggle_dev');
        });
    }
    if (dom.btnNavPalette) {
        dom.btnNavPalette.addEventListener('click', async () => {
            await applyPanelAction('toggle_palette');
        });
    }
    if (dom.btnNavTheme) {
        dom.btnNavTheme.addEventListener('click', async () => {
            state.currentStyle = await invoke('toggle_theme_style_cmd', { style: state.currentStyle });
            applyColors(state.currentStyle);
            await invoke('save_style_config', { config: state.currentStyle });
        });
    }

    // palette components ( 綁定 )
    if (dom.paletteTargetType) {
        dom.paletteTargetType.addEventListener('change', () => {
            const isSpecific = dom.paletteTargetType.value === 'specific';
            const groupGlobal = document.getElementById('group-global');
            const groupSpecific = document.getElementById('group-specific');
            if (groupGlobal) groupGlobal.classList.toggle('hidden', isSpecific);
            if (groupSpecific) groupSpecific.classList.toggle('hidden', !isSpecific);
            if (dom.paletteTargetItem) dom.paletteTargetItem.value = isSpecific ? 'btn-translate' : 'dark_bg';
            updatePaletteValue();
        });
    }
    if (dom.paletteTargetItem) dom.paletteTargetItem.addEventListener('change', updatePaletteValue);
    if (dom.paletteProperty) dom.paletteProperty.addEventListener('change', updatePaletteValue);

    const debouncedSavePalette = debounce(async () => {
        await invoke('save_style_config', { config: state.currentStyle });
    }, 400);

    if (dom.paletteColor) {
        dom.paletteColor.addEventListener('input', async () => {
            state.currentStyle = await invoke('apply_palette_mutation_cmd', {
                style: state.currentStyle,
                input: {
                    target_type: dom.paletteTargetType ? dom.paletteTargetType.value : 'global',
                    target_item: dom.paletteTargetItem ? dom.paletteTargetItem.value : 'dark_bg',
                    property: dom.paletteProperty ? dom.paletteProperty.value : 'bg',
                    color_hex: dom.paletteColor.value,
                    number_value: null,
                },
            });
            applyColors(state.currentStyle);
            debouncedSavePalette();
        });
    }

    if (dom.paletteNumber) {
        dom.paletteNumber.addEventListener('input', async () => {
            state.currentStyle = await invoke('apply_palette_mutation_cmd', {
                style: state.currentStyle,
                input: {
                    target_type: dom.paletteTargetType ? dom.paletteTargetType.value : 'global',
                    target_item: dom.paletteTargetItem ? dom.paletteTargetItem.value : 'dark_bg',
                    property: dom.paletteProperty ? dom.paletteProperty.value : 'rounding',
                    color_hex: null,
                    number_value: parseFloat(dom.paletteNumber.value) || 0,
                },
            });
            applyColors(state.currentStyle);
            debouncedSavePalette();
        });
    }

    // 6. 清除覆寫事件
    if (dom.btnPaletteClearItem) {
        dom.btnPaletteClearItem.addEventListener('click', async () => {
            if (!dom.paletteTargetType || dom.paletteTargetType.value !== 'specific' || !dom.paletteTargetItem) return;
            const target = dom.paletteTargetItem.value;

            try {
                state.currentStyle = await invoke('clear_palette_override_cmd', {
                    style: state.currentStyle,
                    target,
                });

                updatePaletteValue();
                applyColors(state.currentStyle);
                await invoke('save_style_config', { config: state.currentStyle });

                const targetName = dom.paletteTargetItem.options[dom.paletteTargetItem.selectedIndex].text;
                appendLog(state.currentLabels.status_palette_clear_item.replace('{}', targetName));
            } catch (e) {
                console.error(e);
            }
        });
    }

    // 7. 初始化面板展開狀態 (解決初次點擊無反應或誤開啟問題)
    updatePanelVisibility();

    if (dom.btnRestoreApi) dom.btnRestoreApi.addEventListener('click', restoreDefaultConfig);
    if (dom.btnRestoreDev) dom.btnRestoreDev.addEventListener('click', restoreDevDefaults);
    if (dom.btnRestorePalette) dom.btnRestorePalette.addEventListener('click', restoreDefaultStyle);
});

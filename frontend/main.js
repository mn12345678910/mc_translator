// frontend/main.js
import { state } from './modules/state.js';
import { debounce, appendLog } from './modules/utils.js';
import { loadUiLangs, updateUiLanguage, updateToggleStateLabel } from './modules/i18n.js';
import {
    loadConfig,
    saveConfig,
    toggleOllamaGroup,
    toggleApiKeyVisibility,
    validateCanTranslate,
} from './modules/config.js';
import { loadStyle, saveStyle, applyColors, updatePaletteValue } from './modules/style.js';
import { initDictionary, loadDictionary } from './modules/dictionary.js';
import { initTranslation } from './modules/translation.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };

document.addEventListener('DOMContentLoaded', async () => {
    // 1. 初始化資料載入
    await loadConfig();
    await loadUiLangs();
    await loadStyle();
    await loadDictionary();

    if (window.__TAURI__) {
        invoke('show_window');
    }

    // 2. 初始化子模組事件綁定
    initDictionary();
    initTranslation();

    // 3. 基礎按鈕點擊綁定 ( browse 等 )
    const btnBrowseFile = document.getElementById('btn-browse-file');
    const btnBrowseDir = document.getElementById('btn-browse-dir');
    const btnBrowseOutput = document.getElementById('btn-browse-output');
    const btnBrowseOutputOpen = document.getElementById('btn-browse-output-open');
    const inputPath = document.getElementById('input-path');
    const outputDir = document.getElementById('output-dir');

    async function browsePath(type, targetEl) {
        try {
            const path = await invoke('open_path_dialog', { diagType: type });
            if (path && targetEl) targetEl.value = path;
        } catch (e) {
            const mask = state.currentLabels.status_browse_path_failed;
            appendLog(mask.replace('{}', state.currentLabels[e] || e));
        }
    }

    if (btnBrowseFile) btnBrowseFile.addEventListener('click', () => browsePath('file', inputPath));
    if (btnBrowseDir) btnBrowseDir.addEventListener('click', () => browsePath('dir', inputPath));
    if (btnBrowseOutput) btnBrowseOutput.addEventListener('click', () => browsePath('dir', outputDir));

    if (btnBrowseOutputOpen) {
        btnBrowseOutputOpen.addEventListener('click', async () => {
            const target = outputDir ? outputDir.value.trim() : './LLMTranslator';
            try {
                await invoke('open_folder', { path: target });
            } catch (e) {
                console.error(e);
            }
        });
    }

    // 4. 下拉選單與防抖自動儲存
    const apiProvider = document.getElementById('api-provider');
    if (apiProvider) {
        apiProvider.addEventListener('change', async () => {
            toggleOllamaGroup();
            toggleApiKeyVisibility();
            const loadModelsModule = await import('./modules/config.js').then((m) => m.loadModels);
            await loadModelsModule();
            validateCanTranslate();
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
        'ollama-url',
        'batch-size',
        'batch-max-chars',
        'timeout-sec',
        'user-prompt',
        'system-prompt',
    ];
    configInputs.forEach((id) => {
        const el = document.getElementById(id);
        if (el) el.addEventListener('input', debouncedSaveConfig);
    });

    const styleInputs = ['font-size', 'btn-rounding-value', 'pulse-speed'];
    styleInputs.forEach((id) => {
        const el = document.getElementById(id);
        if (el) el.addEventListener('input', debouncedSaveStyle);
    });

    const configSelects = [
        'pack-format',
        'chk-glossary-priority',
        'chk-skip-json',
        'chk-skip-js',
        'chk-skip-jar',
        'chk-skip-book',
        'chk-llm-log',
    ];
    configSelects.forEach((id) => {
        const el = document.getElementById(id);
        if (el) {
            el.addEventListener('change', () => {
                if (id.startsWith('chk-')) {
                    updateToggleStateLabel(id, el.checked);
                }
                debouncedSaveConfig();
            });
        }
    });

    const styleSelects = ['chk-btn-rounding', 'chk-pulse'];
    styleSelects.forEach((id) => {
        const el = document.getElementById(id);
        if (el) el.addEventListener('change', debouncedSaveStyle);
    });

    const uiLang = document.getElementById('ui-lang');
    if (uiLang) {
        uiLang.addEventListener('change', async () => {
            state.currentConfig.ui_lang = uiLang.value;
            document.documentElement.lang = uiLang.value.replace('_', '-');
            await invoke('save_config', { config: state.currentConfig });
            await updateUiLanguage();
        });
    }

    const targetLang = document.getElementById('target-lang');
    if (targetLang) {
        targetLang.addEventListener('change', async () => {
            const lang = targetLang.value;
            try {
                const labels = await invoke('get_i18n_labels_cmd', { lang: lang });
                const userPrompt = document.getElementById('user-prompt');
                const systemPrompt = document.getElementById('system-prompt');
                if (userPrompt && labels.default_user_prompt) {
                    userPrompt.value = labels.default_user_prompt;
                }
                if (systemPrompt && labels.default_system_prompt) {
                    systemPrompt.value = labels.default_system_prompt;
                }
                debouncedSaveConfig();
            } catch (e) {
                console.error('載入預設 Prompts 失敗:', e);
            }
        });
    }

    // 5. 導覽面板控制
    const btnNavApi = document.getElementById('btn-nav-api');
    const btnNavDev = document.getElementById('btn-nav-dev');
    const btnNavPalette = document.getElementById('btn-nav-palette');
    const btnNavTheme = document.getElementById('btn-nav-theme');
    const panelApi = document.querySelector('.api-settings');
    const panelDev = document.querySelector('.developer-settings');
    const panelTheme = document.querySelector('.theme-settings');

    function updatePanelVisibility() {
        if (panelApi) panelApi.style.display = state.currentConfig.show_api_settings ? 'block' : 'none';
        if (panelDev) panelDev.style.display = state.currentConfig.show_developer_mode ? 'block' : 'none';
        if (panelTheme) panelTheme.style.display = state.currentStyle.show_palette_settings ? 'block' : 'none';
    }

    if (btnNavApi) {
        btnNavApi.addEventListener('click', async () => {
            state.currentConfig.show_api_settings = !state.currentConfig.show_api_settings;
            if (state.currentConfig.show_api_settings) {
                state.currentConfig.show_developer_mode = false;
                state.currentStyle.show_palette_settings = false;
            }
            updatePanelVisibility();
            await invoke('save_config', { config: state.currentConfig });
        });
    }
    if (btnNavDev) {
        btnNavDev.addEventListener('click', async () => {
            state.currentConfig.show_developer_mode = !state.currentConfig.show_developer_mode;
            if (state.currentConfig.show_developer_mode) {
                state.currentConfig.show_api_settings = false;
                state.currentStyle.show_palette_settings = false;
            }
            updatePanelVisibility();
            await invoke('save_config', { config: state.currentConfig });
        });
    }
    if (btnNavPalette) {
        btnNavPalette.addEventListener('click', async () => {
            state.currentStyle.show_palette_settings = !state.currentStyle.show_palette_settings;
            if (state.currentStyle.show_palette_settings) {
                state.currentConfig.show_api_settings = false;
                state.currentConfig.show_developer_mode = false;
            }
            updatePanelVisibility();
            await invoke('save_config', { config: state.currentConfig });
        });
    }
    if (btnNavTheme) {
        btnNavTheme.addEventListener('click', async () => {
            state.currentStyle.theme = state.currentStyle.theme === 'dark' ? 'light' : 'dark';
            applyColors(state.currentStyle);
            await invoke('save_style_config', { config: state.currentStyle });
        });
    }

    // palette components ( 綁定 )
    const paletteTargetType = document.getElementById('palette-target-type');
    const paletteTargetItem = document.getElementById('palette-target-item');
    const paletteProperty = document.getElementById('palette-property');
    const paletteRounding = document.getElementById('palette-rounding');
    const paletteColor = document.getElementById('palette-color');

    if (paletteTargetType) {
        paletteTargetType.addEventListener('change', () => {
            const isSpecific = paletteTargetType.value === 'specific';
            const groupGlobal = document.getElementById('group-global');
            const groupSpecific = document.getElementById('group-specific');
            if (groupGlobal) groupGlobal.style.display = isSpecific ? 'none' : 'block';
            if (groupSpecific) groupSpecific.style.display = isSpecific ? 'block' : 'none';
            if (paletteTargetItem) paletteTargetItem.value = isSpecific ? 'btn-translate' : 'dark_bg';
            updatePaletteValue();
        });
    }
    if (paletteTargetItem) paletteTargetItem.addEventListener('change', updatePaletteValue);
    if (paletteProperty) paletteProperty.addEventListener('change', updatePaletteValue);

    const debouncedSavePalette = debounce(async () => {
        await invoke('save_style_config', { config: state.currentStyle });
    }, 400);

    if (paletteColor) {
        paletteColor.addEventListener('input', () => {
            const isSpecific = paletteTargetType ? paletteTargetType.value === 'specific' : false;
            const target = paletteTargetItem ? paletteTargetItem.value : 'dark_bg';
            const prop = paletteProperty ? paletteProperty.value : 'bg';
            const hex = paletteColor.value;
            if (!hex.startsWith('#')) return;
            const bigint = parseInt(hex.slice(1), 16);
            const rgb = [(bigint >> 16) & 255, (bigint >> 8) & 255, bigint & 255];

            if (!isSpecific) {
                state.currentStyle[target] = rgb;
            } else {
                if (!state.currentStyle.instance_overrides) state.currentStyle.instance_overrides = {};
                if (!state.currentStyle.instance_overrides[target]) state.currentStyle.instance_overrides[target] = {};
                state.currentStyle.instance_overrides[target][prop] = rgb;
            }
            applyColors(state.currentStyle);
            debouncedSavePalette();
        });
    }

    if (paletteRounding) {
        paletteRounding.addEventListener('input', () => {
            const isSpecific = paletteTargetType ? paletteTargetType.value === 'specific' : false;
            const target = paletteTargetItem ? paletteTargetItem.value : 'btn-translate';
            const val = parseFloat(paletteRounding.value) || 0;
            if (isSpecific) {
                if (!state.currentStyle.instance_overrides) state.currentStyle.instance_overrides = {};
                if (!state.currentStyle.instance_overrides[target]) state.currentStyle.instance_overrides[target] = {};
                state.currentStyle.instance_overrides[target].rounding = val;
            }
            applyColors(state.currentStyle);
            debouncedSavePalette();
        });
    }

    // 6. 恢復預設值與清除覆寫事件
    const btnRestoreConfig = document.getElementById('btn-restore-config');
    const btnRestoreStyle = document.getElementById('btn-restore-style');
    const btnPaletteClearItem = document.getElementById('btn-palette-clear-item');

    if (btnRestoreConfig) {
        btnRestoreConfig.addEventListener('click', async () => {
            const confirmMsg = state.currentLabels.confirm_restore_text;
            if (confirm(confirmMsg)) {
                try {
                    const defaultConfig = await invoke('get_default_config');
                    await invoke('save_config', { config: defaultConfig });
                    await loadConfig();
                    appendLog(state.currentLabels.status_restore_config_success);
                } catch (e) {
                    appendLog(
                        (state.currentLabels.status_restore_config_failed).replace('{}', e)
                    );
                }
            }
        });
    }

    if (btnRestoreStyle) {
        btnRestoreStyle.addEventListener('click', async () => {
            const confirmMsg = state.currentLabels.confirm_restore_text;
            if (confirm(confirmMsg)) {
                try {
                    const defaultStyle = await invoke('get_default_style_config');
                    await invoke('save_style_config', { config: defaultStyle });
                    await loadStyle();
                    appendLog(state.currentLabels.status_restore_style_success);
                } catch (e) {
                    appendLog(
                        (state.currentLabels.status_restore_style_failed).replace('{}', e)
                    );
                }
            }
        });
    }

    if (btnPaletteClearItem) {
        btnPaletteClearItem.addEventListener('click', async () => {
            if (!paletteTargetType || paletteTargetType.value !== 'specific' || !paletteTargetItem) return;
            const target = paletteTargetItem.value;

            try {
                if (state.currentStyle.instance_overrides && state.currentStyle.instance_overrides[target]) {
                    delete state.currentStyle.instance_overrides[target];
                }

                updatePaletteValue();
                applyColors(state.currentStyle);
                await invoke('save_style_config', { config: state.currentStyle });

                const targetName = paletteTargetItem.options[paletteTargetItem.selectedIndex].text;
                appendLog(
                    (state.currentLabels.status_palette_clear_item).replace('{}', targetName)
                );
            } catch (e) {
                console.error(e);
            }
        });
    }
});

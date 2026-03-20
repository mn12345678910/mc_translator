const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

function debounce(fn, delay) {
    let timer = null;
    return function(...args) {
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, args), delay);
    };
}

document.addEventListener('DOMContentLoaded', async () => {
    // [NEW] 全局翻譯載具，供各處事件使用
    let currentLabels = {};

    // 🛠️ 基礎控制元件
    const inputPath = document.getElementById('input-path');
    const outputDir = document.getElementById('output-dir'); // [NEW]
    const btnBrowseFile = document.getElementById('btn-browse-file');
    const btnBrowseDir = document.getElementById('btn-browse-dir');
    const btnBrowseOutput = document.getElementById('btn-browse-output'); // [NEW]
    const btnTranslate = document.getElementById('btn-translate');
    const btnPause = document.getElementById('btn-pause');
    const btnResume = document.getElementById('btn-resume');
    const btnStop = document.getElementById('btn-stop');
    
    // 🎛️ 導航列按鈕 [NEW]
    const btnNavApi = document.getElementById('btn-nav-api');
    const btnNavDict = document.getElementById('btn-nav-dict');
    const btnNavPalette = document.getElementById('btn-nav-palette');
    const btnNavTheme = document.getElementById('btn-nav-theme');
    const btnNavDev = document.getElementById('btn-nav-dev');

    // 面板容器
    const panelApi = document.querySelector('.api-settings');
    const panelDev = document.querySelector('.developer-settings');
    const panelTheme = document.querySelector('.theme-settings');

    // ⚙️ API 參數元件
    const apiProvider = document.getElementById('api-provider');
    const apiKey = document.getElementById('api-key');
    const selectedModel = document.getElementById('selected-model');
    const ollamaUrl = document.getElementById('ollama-url');
    const ollamaUrlGroup = document.getElementById('ollama-url-group');
    const batchSize = document.getElementById('batch-size');
    const batchMaxChars = document.getElementById('batch-max-chars');
    const timeoutSec = document.getElementById('timeout-sec');
    const packFormat = document.getElementById('pack-format'); // [NEW] <select>
    const glossaryPriority = document.getElementById('glossary-priority'); // [NEW]
    const uiLang = document.getElementById('ui-lang'); // [NEW]
    const btnSaveConfig = document.getElementById('btn-save-config');

    // 📝 Prompts 元件 [NEW]
    const systemPrompt = document.getElementById('system-prompt');
    const userPrompt = document.getElementById('user-prompt');

    // 🔧 開發人員過濾器
    const chkSkipJson = document.getElementById('chk-skip-json');
    const chkSkipJs = document.getElementById('chk-skip-js');
    const chkSkipJar = document.getElementById('chk-skip-jar');
    const chkSkipBook = document.getElementById('chk-skip-book');
    const chkLlmLog = document.getElementById('chk-llm-log');

    // 🎨 調色盤與佈局元組
    const colorBg = document.getElementById('color-bg');
    const colorText = document.getElementById('color-text');
    const colorBtnBg = document.getElementById('color-btn-bg');
    const colorBtnText = document.getElementById('color-btn-text');
    const fontSize = document.getElementById('font-size'); // [NEW]
    const chkBtnRounding = document.getElementById('chk-btn-rounding');
    const btnRoundingValue = document.getElementById('btn-rounding-value');
    const chkPulse = document.getElementById('chk-pulse');
    const pulseSpeed = document.getElementById('pulse-speed');
    const btnSaveStyle = document.getElementById('btn-save-style');

    // 📊 日誌與狀態元組
    const statusText = document.getElementById('status-text');
    const progressBar = document.getElementById('progress-bar');
    const logOutput = document.getElementById('log-output');

    let currentConfig = {};
    let currentStyle = {};

    // --- 🌍 1. 載入介面語言列表 ---
    async function loadUiLangs() {
        try {
            const langs = await invoke('get_available_langs');
            uiLang.innerHTML = '';
            const allLangs = Array.from(new Set([...langs, 'zh_tw', 'zh_cn', 'en_us', 'ja_jp']));
            allLangs.forEach(l => {
                const opt = document.createElement('option');
                opt.value = l;
                opt.textContent = l === 'zh_tw' ? (currentLabels.lang_zh_tw || '繁體中文 (zh_tw)') : 
                                  l === 'en_us' ? (currentLabels.lang_en_us || 'English (en_us)') : 
                                  l === 'zh_cn' ? (currentLabels.lang_zh_cn || '简体中文 (zh_cn)') : 
                                  l === 'ja_jp' ? (currentLabels.lang_ja_jp || '日本語 (ja_jp)') : l;
                uiLang.appendChild(opt);
            });
        } catch (e) {
            console.error('無法載入語言清單', e);
        }
    }
    await loadUiLangs();
    await loadConfig();
    await loadStyle();

    // [NEW] 延遲顯示機制：樣式與設定加載完畢後才顯示，防白屏閃爍
    if (window.__TAURI__) {
        invoke('show_window');
    }

    // 已將殘缺版 updateUiLanguage 移除，統一使用下方的完整版。

    // --- ⚙️ 2. 參數雙向綁定與保存 ---
    async function loadConfig() {
        try {
            const config = await invoke('get_config');
            currentConfig = config;

            apiProvider.value = config.api_provider || 'Gemini';
            
            // 修正: 從獨立指令載入 API Key 防止被 skip 覆蓋
            const savedKey = await invoke('get_api_key_cmd');
            apiKey.value = savedKey || '';
            ollamaUrl.value = config.ollama_url || 'http://localhost:11434';
            batchSize.value = config.batch_size || 150;
            batchMaxChars.value = config.batch_max_chars || 3500;
            timeoutSec.value = config.timeout || 60;
            packFormat.value = config.pack_format ? config.pack_format.toString() : '15';
            glossaryPriority.value = config.glossary_priority || 'official'; // [NEW]
            uiLang.value = config.ui_lang || 'zh_tw';
            outputDir.value = config.output_dir || ''; // [SYNC]

            systemPrompt.value = config.system_prompt || '';
            userPrompt.value = config.user_prompt || '';

            chkSkipJson.checked = config.skip_json || false;
            chkSkipJs.checked = config.skip_js || false;
            chkSkipJar.checked = config.skip_jar || false;
            chkSkipBook.checked = config.skip_book || false;
            chkLlmLog.checked = config.enable_llm_log || false;

            // 🛠️ 控制面板切換 [NEW]
            updatePanelVisibility(config.show_api_settings, config.show_developer_mode, currentStyle.show_palette_settings);

            toggleOllamaGroup();
            await loadModels();
            if (config.model) {
                selectedModel.value = config.model;
            }
            // 🌍 動態更新語言
            updateUiLanguage();
            toggleApiKeyVisibility();
            validateCanTranslate();
        } catch (e) {
            if (typeof appendLog === 'function') {
                const mask = currentLabels.status_load_config_failed || '❌ 載入配置失敗: {}';
                appendLog(mask.replace('{}', currentLabels[e] || e));
            }
        }
    }


    async function saveConfig() {
        try {
            currentConfig.api_provider = apiProvider.value;
            // 修正: 儲存至獨立 Keyring 指令
            await invoke('save_api_key_cmd', { key: apiKey.value });
            currentConfig.model = selectedModel.value;
            currentConfig.ollama_url = ollamaUrl.value;
            currentConfig.batch_size = parseInt(batchSize.value);
            currentConfig.batch_max_chars = parseInt(batchMaxChars.value);
            currentConfig.timeout = parseInt(timeoutSec.value);
            currentConfig.pack_format = parseInt(packFormat.value);
            currentConfig.glossary_priority = glossaryPriority.value; // [NEW]
            currentConfig.ui_lang = uiLang.value;
            currentConfig.output_dir = outputDir.value; // [SYNC]
            currentConfig.path = inputPath.value; // [NEW]

            currentConfig.system_prompt = systemPrompt.value;
            currentConfig.user_prompt = userPrompt.value;

            currentConfig.skip_json = chkSkipJson.checked;
            currentConfig.skip_js = chkSkipJs.checked;
            currentConfig.skip_jar = chkSkipJar.checked;
            currentConfig.skip_book = chkSkipBook.checked;
            currentConfig.enable_llm_log = chkLlmLog.checked;

            await invoke('save_config', { config: currentConfig });
            appendLog(currentLabels.status_save_config_success || '✅ 核心參數儲存成功！');
            updateUiLanguage(); // [NEW] 儲存後也重新整理語言
        } catch (e) {
            const mask = currentLabels.status_save_config_failed || '❌ 儲存配置失敗: {}';
            appendLog(mask.replace('{}', currentLabels[e] || e));
        }
    }


    // --- 🎨 3. 調色盤與字體縮放 ---
    async function loadStyle() {
        try {
            const style = await invoke('get_style_config');
            currentStyle = style;

            const isDark = style.theme !== 'light';
            colorBg.value = rgbToHex(isDark ? style.dark_bg : style.light_bg);
            colorText.value = rgbToHex(isDark ? style.dark_text : style.light_text);
            colorBtnBg.value = rgbToHex(isDark ? style.dark_btn_bg : style.light_btn_bg);
            colorBtnText.value = rgbToHex(isDark ? style.dark_btn_text : style.light_btn_text);
            
            if (style.font_size) {
                fontSize.value = style.font_size;
                document.documentElement.style.setProperty('--font-size', style.font_size + 'px');
            }
            // 圓角與動畫
            chkBtnRounding.checked = style.btn_rounding_enabled ?? true;
            btnRoundingValue.value = style.btn_rounding_value ?? 4.0;
            chkPulse.checked = style.progress_pulse_enabled ?? true;
            pulseSpeed.value = style.progress_pulse_speed ?? 1.0;

            applyColors(style);
        } catch (e) {
            console.error(e);
        }
    }


    async function saveStyle() {
        try {
            const isDark = currentStyle.theme !== 'light';
            const rgb = hexToRgb(colorBg.value);
            if (isDark) currentStyle.dark_bg = rgb; else currentStyle.light_bg = rgb;
            const txt = hexToRgb(colorText.value);
            if (isDark) currentStyle.dark_text = txt; else currentStyle.light_text = txt;
            const bbg = hexToRgb(colorBtnBg.value);
            if (isDark) currentStyle.dark_btn_bg = bbg; else currentStyle.light_btn_bg = bbg;
            const btxt = hexToRgb(colorBtnText.value);
            if (isDark) currentStyle.dark_btn_text = btxt; else currentStyle.light_btn_text = btxt;

            currentStyle.font_size = parseFloat(fontSize.value);
            currentStyle.btn_rounding_enabled = chkBtnRounding.checked;
            currentStyle.btn_rounding_value = parseFloat(btnRoundingValue.value);
            currentStyle.progress_pulse_enabled = chkPulse.checked;
            currentStyle.progress_pulse_speed = parseFloat(pulseSpeed.value);

            await invoke('save_style_config', { config: currentStyle });
            appendLog(currentLabels.status_save_style_success || '🎨 調色盤與佈局保存成功！');
            applyColors(currentStyle);
            document.documentElement.style.setProperty('--font-size', fontSize.value + 'px');

        } catch (e) {
            const mask = currentLabels.status_save_style_failed || '❌ 保存樣式失敗: {}';
            appendLog(mask.replace('{}', currentLabels[e] || e));
        }
    }


    // --- 🎮 4. 面板熔斷鎖狀態機 ---
    function setRunningState(isRunning) {
        // btnTranslate.disabled = isRunning; // 停用改為隱藏
        
        // 🔒 鎖定所有輸入框與面板以防誤觸 (100% 復刻熔斷閥)
        const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
        inputs.forEach(el => el.disabled = isRunning);
        
        if (isRunning) {
            btnTranslate.style.display = 'none';
            btnPause.style.display = 'inline-block';
            btnStop.style.display = 'inline-block';
            btnPause.textContent = currentLabels.btn_pause || '⏸️ 暫停';
        } else {
            btnTranslate.style.display = 'inline-block';
            btnPause.style.display = 'none';
            btnResume.style.display = 'none';
            btnStop.style.display = 'none';
        }
    }

    // --- 📂 5. Native 瀏覽對話框連鎖 ---
    async function browsePath(type, targetEl) {
        try {
            const path = await invoke('open_path_dialog', { diagType: type });
            if (path) targetEl.value = path;
        } catch (e) {
            const mask = currentLabels.status_browse_path_failed || '❌ 瀏覽路徑失敗: {}';
            appendLog(mask.replace('{}', currentLabels[e] || e));
        }
    }

    btnBrowseFile.addEventListener('click', () => browsePath('file', inputPath));
    btnBrowseDir.addEventListener('click', () => browsePath('dir', inputPath));
    btnBrowseOutput.addEventListener('click', () => browsePath('dir', outputDir)); // [NEW]
    
    const btnBrowseOutputOpen = document.getElementById('btn-browse-output-open');
    if (btnBrowseOutputOpen) {
        btnBrowseOutputOpen.addEventListener('click', async () => {
            const target = outputDir.value.trim() || './LLMTranslator';
            try {
                await invoke('open_folder', { path: target });
            } catch (e) {
                const mask = currentLabels.status_open_dir_failed || '❌ 無法打開資料夾: {}';
                appendLog(mask.replace('{}', currentLabels[e] || e));
            }
        });
    }

    // --- 其他邏輯 ---
    async function loadModels() {
        const provider = apiProvider.value;
        selectedModel.innerHTML = `<option value="">${currentLabels.label_loading_models || '載入中...'}</option>`;
        try {
            const models = await invoke('get_models_from_provider', { provider });
            selectedModel.innerHTML = '';
            models.forEach(m => {
                const opt = document.createElement('option');
                opt.value = m; opt.textContent = m;
                selectedModel.appendChild(opt);
            });
        } catch (e) {
            selectedModel.innerHTML = `<option value="">${currentLabels.label_no_models || '(無可用模型)'}</option>`;
        }
    }

    function toggleOllamaGroup() {
        ollamaUrlGroup.style.display = apiProvider.value === 'Ollama' ? 'block' : 'none';
    }

    function toggleApiKeyVisibility() {
        const apiKeyGroup = document.getElementById('api-key-group');
        if (apiKeyGroup) {
            const noKeyProviders = ['Ollama', 'Google Free', '無'];
            apiKeyGroup.style.display = noKeyProviders.includes(apiProvider.value) ? 'none' : 'block';
        }
    }

    function validateCanTranslate() {
        btnTranslate.disabled = !selectedModel.value && apiProvider.value !== 'Google Free' && apiProvider.value !== 'Ollama';
    }

    apiProvider.addEventListener('change', async () => { 
        toggleOllamaGroup(); 
        toggleApiKeyVisibility();
        await loadModels(); 
        validateCanTranslate();
    });
    selectedModel.addEventListener('change', validateCanTranslate);
    btnSaveConfig.addEventListener('click', saveConfig);
    btnSaveStyle.addEventListener('click', saveStyle);

    if (uiLang) {
        uiLang.addEventListener('change', async () => {
            currentConfig.ui_lang = uiLang.value;
            await invoke('save_config', { config: currentConfig });
            await updateUiLanguage();
            const mask = currentLabels.status_ui_lang_changed || '🌍 介面語言已變更為：{}';
            appendLog(mask.replace('{}', uiLang.value));
        });
    }

    // 🎛️ 面板 UI 控制與排斥邏輯
    function updatePanelVisibility(showApi, showDev, showPalette) {
        panelApi.style.display = showApi ? 'block' : 'none';
        panelDev.style.display = showDev ? 'block' : 'none';
        panelTheme.style.display = showPalette ? 'block' : 'none';
    }

    if (btnNavApi) {
        btnNavApi.addEventListener('click', async () => {
            currentConfig.show_api_settings = !currentConfig.show_api_settings;
            if (currentConfig.show_api_settings) {
                currentConfig.show_developer_mode = false;
                currentStyle.show_palette_settings = false;
            }
            updatePanelVisibility(currentConfig.show_api_settings, currentConfig.show_developer_mode, currentStyle.show_palette_settings);
            await invoke('save_config', { config: currentConfig });
        });
    }

    if (btnNavDev) {
        btnNavDev.addEventListener('click', async () => {
            currentConfig.show_developer_mode = !currentConfig.show_developer_mode;
            if (currentConfig.show_developer_mode) {
                currentConfig.show_api_settings = false;
                currentStyle.show_palette_settings = false;
            }
            updatePanelVisibility(currentConfig.show_api_settings, currentConfig.show_developer_mode, currentStyle.show_palette_settings);
            await invoke('save_config', { config: currentConfig });
        });
    }

    if (btnNavPalette) {
        btnNavPalette.addEventListener('click', async () => {
             currentStyle.show_palette_settings = !currentStyle.show_palette_settings;
             if (currentStyle.show_palette_settings) {
                 currentConfig.show_api_settings = false;
                 currentConfig.show_developer_mode = false;
             }
             updatePanelVisibility(currentConfig.show_api_settings, currentConfig.show_developer_mode, currentStyle.show_palette_settings);
             await invoke('save_config', { config: currentConfig });
             // [Optional] 記錄 Palette 面板是否展開，為簡化我們把這狀態借用 config
        });
    }

    if (btnNavTheme) {
        btnNavTheme.addEventListener('click', async () => {
            currentStyle.theme = currentStyle.theme === 'dark' ? 'light' : 'dark';
            applyColors(currentStyle); // 立刻變換視覺！
            await invoke('save_style_config', { config: currentStyle });
            const mask = currentLabels.status_theme_changed || '🌓 主題已切換為：{}';
            const themeName = currentStyle.theme === 'dark' ? (currentLabels.mode_dark || '暗色') : (currentLabels.mode_light || '亮色');
            appendLog(mask.replace('{}', themeName));
        });
    }



    async function updateUiLanguage() {
        try {
            // [NEW] 記錄更新語言狀態
            console.log(`🌍 嘗試切換介面語言為: ${uiLang ? uiLang.value : 'unknown'}`);
            const labels = await invoke('get_i18n_labels', { lang: uiLang ? uiLang.value : undefined });
            console.log(`🌍 已獲取到 ${labels ? Object.keys(labels).length : 0} 個翻譯欄位`);
            if (!labels) return;
            currentLabels = { ...labels }; // [NEW] 備份至全域指標

            // [NEW] 連動更新 <html lang="..."> 屬性
            if (uiLang && uiLang.value) {
                document.documentElement.lang = uiLang.value.replace('_', '-');
            }

            const titleNode = document.querySelector('h1 span') || document.querySelector('h1');
            if (titleNode && labels.app_title) titleNode.textContent = labels.app_title;

            const mapping = {
                'btn-browse-file': labels.btn_select_file,
                'btn-browse-dir': labels.btn_select_folder,
                'btn-browse-output': labels.btn_output_dir,
                'btn-browse-output-open': labels.btn_open_output,
                'btn-translate': labels.btn_run_trans,
                'btn-pause': labels.btn_pause,
                'btn-stop': labels.btn_stop,
                'btn-resume': labels.btn_resume,
                'btn-save-config': labels.btn_save_config,
                'btn-restore-config': labels.btn_restore_defaults,
                'btn-save-style': labels.btn_save_style,
                'btn-restore-style': labels.btn_restore_defaults,
                // [NEW] 標題文字
                'header-api-settings': labels.header_api_settings,
                'header-palette': labels.header_palette,
                'header-dev-mode': labels.header_dev_mode,
                'header-dict-mgr': labels.header_dict_mgr,
                'btn-dict-clear': labels.btn_clear_all,
                'btn-dict-import': labels.btn_import,
                'btn-dict-export': labels.btn_export,
                'btn-dict-replace': labels.btn_replace,
                'btn-dict-add': labels.btn_add,
                'tab-user': labels.glossary_tab_user || '使用者詞庫',
                'tab-official': labels.glossary_tab_official || '官方推論',
                'page-prev': labels.btn_page_prev || '上一頁',
                'page-next': labels.btn_page_next || '下一頁',
                'label-items': labels.label_items,
                'label-files': labels.label_files,
                'btn-palette-clear-item': labels.btn_restore_defaults || '🗑 清除元件覆寫'
            };

            for (const [id, txt] of Object.entries(mapping)) {
                const el = document.getElementById(id);
                if (el && txt) el.textContent = txt;
            }

            // --- 🏷️ 2. 自動媒合所有 <label[for]> 及 <span[id]> ---
            document.querySelectorAll('label[for]').forEach(el => {
                const forId = el.getAttribute('for');
                const underscored = forId.replace(/-/g, '_');
                const key1 = `label_${underscored}`;
                const key2 = underscored;
                
                if (forId === 'api-provider' && labels.label_provider) el.textContent = labels.label_provider;
                else if (forId === 'selected-model' && labels.label_model) el.textContent = labels.label_model;
                else if (forId === 'batch-max-chars' && labels.label_max_chars) el.textContent = labels.label_max_chars;
                else if (forId === 'timeout-sec' && labels.label_timeout) el.textContent = labels.label_timeout;
                else if (forId === 'glossary-priority' && labels.label_glossary_priority) el.textContent = labels.label_glossary_priority;
                else if (forId === 'palette-target-type' && labels.label_palette_target_type) el.textContent = labels.label_palette_target_type;
                else if (forId === 'palette-target-item' && labels.label_palette_target_item) el.textContent = labels.label_palette_target_item;
                else if (forId === 'palette-property' && labels.label_palette_property) el.textContent = labels.label_palette_property;
                else if (forId === 'palette-color' && labels.label_palette_color) el.textContent = labels.label_palette_color;
                else if (forId === 'palette-rounding' && labels.label_palette_rounding) el.textContent = labels.label_palette_rounding;
                else if (forId === 'user-prompt' && labels.label_user_prompt) el.textContent = labels.label_user_prompt;
                else if (forId === 'system-prompt' && labels.label_system_prompt) el.textContent = labels.label_system_prompt;
                else if (forId === 'input-path' && labels.label_input_path) el.textContent = labels.label_input_path;
                else if (labels[key1]) el.textContent = labels[key1];
                else if (labels[key2]) el.textContent = labels[key2];
            });

            document.querySelectorAll('span[id]').forEach(el => {
                const id = el.id;
                const underscored = id.replace(/-/g, '_');
                const key1 = underscored.startsWith('label_') ? underscored : `label_${underscored}`;
                const key2 = underscored;

                if (labels[key1]) el.textContent = labels[key1];
                else if (labels[key2]) el.textContent = labels[key2];
            });

            // --- 🏷️ 3. 自動更新 <select> 選項 (Option) ---
            const optionMapping = {
                'glossary-priority': {
                    'official': labels.glossary_priority_official,
                    'user': labels.glossary_priority_user
                },
                'palette-target-type': {
                    'global': labels.group_batch,
                    'specific': labels.group_specific
                },
                'api-provider': {
                    'Ollama': labels.label_ollama_url ? 'Ollama' : 'Ollama', // keep names
                    '無': labels.label_provider_none || '無'
                },
                'palette-target-item': {
                    'btn-translate': labels.spec_btn_run_trans,
                    'btn-pause': labels.spec_btn_pause,
                    'btn-stop': labels.spec_btn_stop,
                    'btn-browse-file': labels.spec_btn_select_file,
                    'btn-browse-dir': labels.spec_btn_select_folder,
                    'btn-browse-output': labels.spec_btn_output_dir,
                    'btn-browse-output-open': labels.spec_btn_open_output,
                    'user-prompt': labels.label_user_prompt,
                    'system-prompt': labels.label_system_prompt,
                    'input-path': labels.label_input_path,
                    'output-dir': labels.spec_label_output,
                    'dict-dialog': labels.spec_area_dict,
                    'log-output': labels.label_log_area,
                    'progress-bar': labels.spec_progress_current,
                    'batch-progress-bar': labels.spec_progress_total
                },
                'palette-property': {
                    'bg': labels.label_bg_color,
                    'text': labels.label_text_color,
                    'rounding': labels.label_custom_rounding
                },
                'api-provider': {
                    '無': labels.label_provider_none || '無'
                }
            };
            
            for (const [selectId, optionsDict] of Object.entries(optionMapping)) {
                const selectEl = document.getElementById(selectId);
                if (!selectEl) continue;
                for (const [val, txt] of Object.entries(optionsDict)) {
                    if (!txt) continue;
                    const opt = selectEl.querySelector(`option[value="${val}"]`);
                    if (opt) opt.textContent = txt;
                }
            }

            const optgroupMapping = {
                'group-global': labels.group_batch,
                'group-specific': labels.group_specific
            };
            for (const [id, txt] of Object.entries(optgroupMapping)) {
                const el = document.getElementById(id);
                if (el && txt) el.label = txt;
            }

            // [NEW] 補充導覽列按鈕 Title 提示字
            if (labels.spec_btn_nav_settings && btnNavApi) btnNavApi.title = labels.spec_btn_nav_settings;
            if (labels.spec_btn_nav_dict && btnNavDict) btnNavDict.title = labels.spec_btn_nav_dict;
            if (labels.spec_btn_nav_palette && btnNavPalette) btnNavPalette.title = labels.spec_btn_nav_palette;
            if (labels.spec_btn_nav_theme && btnNavTheme) btnNavTheme.title = labels.spec_btn_nav_theme;
            if (labels.spec_btn_nav_dev && btnNavDev) btnNavDev.title = labels.spec_btn_nav_dev;

            // [NEW] 補充面板標題 H2 刷新
            // --- 🏷️ 4. 自動更新 <input[placeholder]> 及 <textarea[placeholder]> ---
            document.querySelectorAll('input[placeholder], textarea[placeholder]').forEach(el => {
                const id = el.id;
                if (!id) return;
                const underscored = id.replace(/-/g, '_');
                const key = `placeholder_${underscored}`;
                
                if (id === 'dict-search' && labels.placeholder_search_terms) el.placeholder = labels.placeholder_search_terms;
                else if (id === 'dict-input-key' && labels.placeholder_dict_key) el.placeholder = labels.placeholder_dict_key;
                else if (id === 'dict-input-value' && labels.placeholder_dict_value) el.placeholder = labels.placeholder_dict_value;
                else if (id === 'input-path' && labels.placeholder_input_path) el.placeholder = labels.placeholder_input_path;
                else if (labels[key]) el.placeholder = labels[key];
            });
        } catch (err) {
            console.error('更新介面語言失敗', err);
        }
    }

    async function saveStyle() {
        if (!currentStyle) return;
        if (chkBtnRounding) currentStyle.btn_rounding_enabled = chkBtnRounding.checked;
        if (btnRoundingValue) currentStyle.btn_rounding_value = parseFloat(btnRoundingValue.value) || 4.0;
        if (chkPulse) currentStyle.progress_pulse_enabled = chkPulse.checked;
        if (pulseSpeed) currentStyle.progress_pulse_speed = parseFloat(pulseSpeed.value) || 1.0;
        const fsInput = document.getElementById('font-size');
        if (fsInput) currentStyle.font_size = parseFloat(fsInput.value) || 15;

        await invoke('save_style_config', { config: currentStyle });
        applyColors(currentStyle); // 確保畫面即時渲染
    }

    // [NEW] 自動儲存綁定 (套用防抖 Debounce 避免頻繁讀寫)
    const debouncedSaveConfig = debounce(async () => { await saveConfig(); }, 500);
    const debouncedSaveStyle = debounce(async () => { await saveStyle(); }, 500);

    // 1. 文字與數字類輸入框 (監聽 input 進行防抖)
    const configInputs = [apiKey, ollamaUrl, batchSize, batchMaxChars, timeoutSec, userPrompt, systemPrompt];
    configInputs.forEach(el => {
        if (el) el.addEventListener('input', debouncedSaveConfig);
    });

    const styleInputs = [document.getElementById('font-size'), btnRoundingValue, pulseSpeed];
    styleInputs.forEach(el => {
        if (el) el.addEventListener('input', debouncedSaveStyle);
    });

    // 2. 下拉選單與勾選框 (監聽 change 即刻儲存)
    const configSelects = [apiProvider, selectedModel, packFormat, glossaryPriority, uiLang, chkSkipJson, chkSkipJs, chkSkipJar, chkSkipBook, chkLlmLog];
    configSelects.forEach(el => {
        if (el) el.addEventListener('change', async () => await saveConfig());
    });

    const styleSelects = [chkBtnRounding, chkPulse];
    styleSelects.forEach(el => {
        if (el) el.addEventListener('change', async () => await saveStyle());
    });

    // [NEW] 獨立綁定：確保下拉選單點選時，<html> 標籤的 lang 屬性物理連動
    if (uiLang) {
        uiLang.addEventListener('change', () => {
            document.documentElement.lang = uiLang.value.replace('_', '-');
        });
    }

    // --- 🎨 3.5 調色盤面板連動控制 ---
    const paletteTargetType = document.getElementById('palette-target-type');
    const paletteTargetItem = document.getElementById('palette-target-item');
    const groupGlobal = document.getElementById('group-global');
    const groupSpecific = document.getElementById('group-specific');
    const paletteColor = document.getElementById('palette-color');

    const paletteProperty = document.getElementById('palette-property');
    const palettePropertyGroup = document.getElementById('palette-property-group');
    const paletteColorGroup = document.getElementById('palette-color-group');
    const paletteRoundingGroup = document.getElementById('palette-rounding-group');
    const paletteRounding = document.getElementById('palette-rounding');
    const labelPaletteColor = document.getElementById('label-palette-color');
    const btnPaletteClearItem = document.getElementById('btn-palette-clear-item');
    const paletteClearGroup = document.getElementById('palette-clear-group');

    if (paletteTargetType && paletteTargetItem) {
        function updatePaletteValue() {
            const isSpecific = paletteTargetType.value === 'specific';
            const target = paletteTargetItem.value;
            const prop = paletteProperty ? paletteProperty.value : 'bg';

            function rgbToHexStr(arr) {
                if (!arr || arr.length < 3) return '#1e1e23';
                return '#' + arr.map(x => x.toString(16).padStart(2, '0')).join('');
            }

            if (paletteClearGroup) paletteClearGroup.style.display = isSpecific ? 'flex' : 'none';

            if (!isSpecific) {
                if (palettePropertyGroup) palettePropertyGroup.style.display = 'none';
                if (paletteColorGroup) paletteColorGroup.style.display = 'block';
                if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'none';

                const color = currentStyle[target];
                if (color && paletteColor) paletteColor.value = rgbToHexStr(color);
            } else {
                if (palettePropertyGroup) palettePropertyGroup.style.display = 'block';
                const override = currentStyle.instance_overrides ? currentStyle.instance_overrides[target] : null;

                if (prop === 'rounding') {
                    if (paletteColorGroup) paletteColorGroup.style.display = 'none';
                    if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'block';
                    if (override && override.rounding !== undefined) {
                        if (paletteRounding) paletteRounding.value = override.rounding;
                    } else {
                        if (paletteRounding) paletteRounding.value = 4;
                    }
                } else {
                    if (paletteColorGroup) paletteColorGroup.style.display = 'block';
                    if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'none';
                    if (labelPaletteColor) labelPaletteColor.textContent = prop === 'bg' ? (currentLabels.label_bg_color || '背景顏色') : (currentLabels.label_text_color || '文字顏色');

                    let color = null;
                    if (override) {
                        color = prop === 'bg' ? override.bg : override.text;
                    }
                    if (color && paletteColor) {
                        paletteColor.value = rgbToHexStr(color);
                    } else {
                        paletteColor.value = '#ffffff';
                    }
                }
            }
        }

        paletteTargetType.addEventListener('change', () => {
            const isSpecific = paletteTargetType.value === 'specific';
            groupGlobal.style.display = isSpecific ? 'none' : 'block';
            groupSpecific.style.display = isSpecific ? 'block' : 'none';
            paletteTargetItem.value = isSpecific ? 'btn-translate' : 'dark_bg';
            updatePaletteValue();
        });

        paletteTargetItem.addEventListener('change', updatePaletteValue);
        if (paletteProperty) paletteProperty.addEventListener('change', updatePaletteValue);

        if (paletteColor) {
            const debouncedSaveStyleConfig = debounce(async () => {
                await invoke('save_style_config', { config: currentStyle });
            }, 400);

            paletteColor.addEventListener('input', () => {
                const isSpecific = paletteTargetType.value === 'specific';
                const target = paletteTargetItem.value;
                const prop = paletteProperty ? paletteProperty.value : 'bg';
                const hex = paletteColor.value;

                function hexToRgbArr(h) {
                    const bigint = parseInt(h.slice(1), 16);
                    return [(bigint >> 16) & 255, (bigint >> 8) & 255, bigint & 255];
                }
                if (!hex.startsWith('#')) return;
                const rgb = hexToRgbArr(hex);

                if (!isSpecific) {
                    currentStyle[target] = rgb;
                } else {
                    if (!currentStyle.instance_overrides) currentStyle.instance_overrides = {};
                    if (!currentStyle.instance_overrides[target]) currentStyle.instance_overrides[target] = {};
                    currentStyle.instance_overrides[target][prop] = rgb;
                }

                applyColors(currentStyle); // 即時更新視覺
                debouncedSaveStyleConfig(); // 延遲存檔
            });
        }

        if (paletteRounding) {
            const debouncedSaveStyleConfig = debounce(async () => {
                await invoke('save_style_config', { config: currentStyle });
            }, 400);

            paletteRounding.addEventListener('input', () => {
                const isSpecific = paletteTargetType.value === 'specific';
                const target = paletteTargetItem.value;
                const val = parseFloat(paletteRounding.value) || 0;

                if (isSpecific) {
                    if (!currentStyle.instance_overrides) currentStyle.instance_overrides = {};
                    if (!currentStyle.instance_overrides[target]) currentStyle.instance_overrides[target] = {};
                    currentStyle.instance_overrides[target].rounding = val;
                }

                applyColors(currentStyle);
                debouncedSaveStyleConfig();
            });
        }

        if (btnPaletteClearItem) {
            btnPaletteClearItem.addEventListener('click', async () => {
                const target = paletteTargetItem.value;
                if (currentStyle.instance_overrides && currentStyle.instance_overrides[target]) {
                    delete currentStyle.instance_overrides[target];
                    applyColors(currentStyle);
                    await invoke('save_style_config', { config: currentStyle });
                    updatePaletteValue(); // 刷新輸入框顯示
                    if (typeof appendLog === 'function') {
                        const mask = currentLabels.status_palette_clear_item || '🎨 已清除元件覆寫: {}';
                        appendLog(mask.replace('{}', target));
                    }
                }
            });
        }
    }

    // --- 🚀 翻譯執行與監聽 ---
    btnTranslate.addEventListener('click', async () => {
        const path = inputPath.value.trim();
        if (!path) { appendLog(currentLabels.status_input_path_empty || '⚠️ 請輸入或選取待翻譯路徑！'); return; }
        setRunningState(true);
        logOutput.innerHTML = '';
        appendLog(currentLabels.status_trans_starting || '🚀 翻譯任務開始發射...');

        try {
        // 同步 UI 狀態至 currentConfig
        currentConfig.api_provider = apiProvider.value;
        currentConfig.model = selectedModel.value;
        currentConfig.ollama_url = ollamaUrl.value;
        currentConfig.batch_size = parseInt(batchSize.value);
        currentConfig.batch_max_chars = parseInt(batchMaxChars.value);
        currentConfig.timeout = parseInt(timeoutSec.value);
        currentConfig.pack_format = parseInt(packFormat.value);
        currentConfig.glossary_priority = glossaryPriority.value;
        currentConfig.user_prompt = userPrompt.value;
        currentConfig.system_prompt = systemPrompt.value;
        currentConfig.skip_json = chkSkipJson.checked;
        currentConfig.skip_js = chkSkipJs.checked;
        currentConfig.skip_jar = chkSkipJar.checked;
        currentConfig.skip_book = chkSkipBook.checked;
        currentConfig.enable_llm_log = chkLlmLog.checked;

        // 修正: 參數對齊後端
            await invoke('start_translation', {
                inputPaths: [path],
                config: currentConfig
            });
            appendLog(currentLabels.status_trans_command_sent || '✅ 任務執行指令送達後端。');
        } catch (e) {
            const mask = currentLabels.status_trans_error || '❌ 執行出錯: {}';
            appendLog(mask.replace('{}', currentLabels[e] || e));
            setRunningState(false);
        }
    });

    btnPause.addEventListener('click', async () => {
        const pauseText = currentLabels.btn_pause || '⏸️ 暫停';
        const resumeText = currentLabels.btn_resume || '▶️ 繼續運行';

        if (btnPause.textContent === pauseText || btnPause.textContent === '⏸️ 暫停') {
            await invoke('pause_translation'); 
            btnPause.textContent = resumeText; 
            appendLog(currentLabels.status_trans_paused || '⏸️ 任務已暫停。面板解鎖，可改動設定。');
            const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
            inputs.forEach(el => el.disabled = false); // 暫停時解除鎖定
        } else {
            await invoke('resume_translation'); 
            btnPause.textContent = pauseText; 
            appendLog(currentLabels.status_trans_resumed || '▶️ 任務已繼續。');
            const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
            inputs.forEach(el => el.disabled = true);
        }
    });

    btnStop.addEventListener('click', async () => { 
        await invoke('stop_translation'); 
        appendLog(currentLabels.status_trans_stopping || '⏹️ 正在送出終止信號...'); 
    });

    // 📻 監聽後端 Event
    listen('translation-progress', (event) => {
        const { current, total, batch_current, batch_total, status } = event.payload;
        
        // 檔案進度 (下層)
        statusText.textContent = `${status} (${batch_current} / ${batch_total})`;
        progressBar.style.width = batch_total > 0 ? `${(batch_current / batch_total) * 100}%` : '0%';
        
        // 條目進度 (上層)
        const batchProgressContainer = document.getElementById('batch-progress-container');
        const batchProgressBar = document.getElementById('batch-progress-bar');
        const batchStatusText = document.getElementById('batch-status-text');

        if (batchProgressContainer && batchProgressBar) {
            if (total > 0) {
                batchProgressContainer.style.display = 'block';
                if (batchStatusText) {
                    batchStatusText.style.display = 'inline';
                    batchStatusText.textContent = `${current} / ${total}`;
                }
                batchProgressBar.style.width = `${(current / total) * 100}%`;
            } else {
                batchProgressContainer.style.display = 'none';
                if (batchStatusText) batchStatusText.style.display = 'none';
                batchProgressBar.style.width = '0%';
            }
        }
    });

    listen('translation-log', (event) => { appendLog(event.payload); });
    listen('translation-status', (event) => { statusText.textContent = event.payload; setRunningState(false); });

    function appendLog(text) {
        const p = document.createElement('p'); 
        p.textContent = `[${new Date().toLocaleTimeString()}] ${text}`;
        if (String(text).includes('❌') || String(text).includes('⚠') || String(text).includes('Error')) {
            p.style.color = '#ff6b6b';
        }
        logOutput.appendChild(p); 
        logOutput.scrollTop = logOutput.scrollHeight;
        if (logOutput.childNodes.length > 500) {
            logOutput.removeChild(logOutput.firstChild);
        }
    }

    function rgbToHex(arr) { if (!arr || arr.length < 3) return '#333333'; return '#' + arr.map(x => x.toString(16).padStart(2, '0')).join(''); }
    function hexToRgb(hex) { const bigint = parseInt(hex.slice(1), 16); return [(bigint >> 16) & 255, (bigint >> 8) & 255, bigint & 255]; }
    function applyColors(style) {
        const isDark = style.theme !== 'light';
        const bg = isDark ? style.dark_bg : style.light_bg;
        const txt = isDark ? style.dark_text : style.light_text;
        const btnBg = isDark ? style.dark_btn_bg : style.light_btn_bg;
        const btnTxt = isDark ? style.dark_btn_text : style.light_btn_text;
        const inputBg = isDark ? style.dark_input_bg : style.light_input_bg;
        const listBg = isDark ? style.dark_list_bg : style.light_list_bg;

        if (bg) document.documentElement.style.setProperty('--bg-color', `rgb(${bg[0]},${bg[1]},${bg[2]})`);
        if (txt) document.documentElement.style.setProperty('--text-color', `rgb(${txt[0]},${txt[1]},${txt[2]})`);
        if (btnBg) document.documentElement.style.setProperty('--btn-bg', `rgb(${btnBg[0]},${btnBg[1]},${btnBg[2]})`);
        if (btnTxt) document.documentElement.style.setProperty('--btn-text', `rgb(${btnTxt[0]},${btnTxt[1]},${btnTxt[2]})`);
        if (inputBg) document.documentElement.style.setProperty('--input-bg', `rgb(${inputBg[0]},${inputBg[1]},${inputBg[2]})`);
        if (listBg) document.documentElement.style.setProperty('--list-bg', `rgb(${listBg[0]},${listBg[1]},${listBg[2]})`);

        if (style.font_size) {
            document.documentElement.style.setProperty('--font-size', `${style.font_size}px`);
        }

        // 套用圓角
        if (style.btn_rounding_enabled !== false) {
            document.documentElement.style.setProperty('--border-radius', `${style.btn_rounding_value ?? 4.0}px`);
        } else {
            document.documentElement.style.setProperty('--border-radius', '0px');
        }

        // 套用進度條脈衝動畫
        if (style.progress_pulse_enabled) {
            const speed = style.progress_pulse_speed ?? 1.0;
            progressBar.style.animation = `pulse ${2.0 / Math.max(0.1, speed)}s infinite`;
        } else {
            progressBar.style.animation = 'none';
        }

        // [NEW] 疊加特定元件覆寫 (instance_overrides)
        if (style.instance_overrides) {
            for (const [id, override] of Object.entries(style.instance_overrides)) {
                const el = document.getElementById(id);
                if (el) {
                    if (override.bg) el.style.backgroundColor = `rgb(${override.bg[0]},${override.bg[1]},${override.bg[2]})`;
                    if (override.text) el.style.color = `rgb(${override.text[0]},${override.text[1]},${override.text[2]})`;
                    if (override.rounding !== undefined) el.style.borderRadius = `${override.rounding}px`;
                }
            }
        }
    }

    function hexToColor(arr) { if (!arr || arr.length < 3) return '#fff'; return `rgb(${arr[0]},${arr[1]},${arr[2]})`; }

    // --- 🌍 6. 恢復預設參數與樣式 ---
    const btnRestoreConfig = document.getElementById('btn-restore-config');
    const btnRestoreStyle = document.getElementById('btn-restore-style');

    if (btnRestoreConfig) {
        btnRestoreConfig.addEventListener('click', async () => {
            if (!confirm(currentLabels.status_restore_config_confirm || '確定要將參數恢復為預設值嗎？')) return;
            try {
                const defaultConfig = await invoke('get_default_config');
                const currentPaths = {
                    api_key: apiKey.value, 
                    output_dir: document.getElementById('output-dir').value
                };
                currentConfig = { ...defaultConfig, ...currentPaths };
                await invoke('save_config', { config: currentConfig });
                appendLog(currentLabels.status_restore_config_success || '✅ 參數已恢復預設！');
                await loadConfig();
            } catch (e) { 
                const mask = currentLabels.status_restore_config_failed || '❌ 恢復參數失敗: {}';
                appendLog(mask.replace('{}', currentLabels[e] || e)); 
            }
        });
    }

    if (btnRestoreStyle) {
        btnRestoreStyle.addEventListener('click', async () => {
            if (!confirm(currentLabels.status_restore_style_confirm || '確定要將外觀佈景恢復為預設嗎？')) return;
            try {
                const defaultStyle = await invoke('get_default_style_config');
                currentStyle = defaultStyle;
                await invoke('save_style_config', { style: currentStyle });
                appendLog(currentLabels.status_restore_style_success || '🎨 外觀已恢復預設！');
                await loadStyle();
            } catch (e) { 
                const mask = currentLabels.status_restore_style_failed || '❌ 恢復樣式失敗: {}';
                appendLog(mask.replace('{}', currentLabels[e] || e)); 
            }
        });
    }

    // --- 📖 7. 字典管理器控制 ---
    const dictDialog = document.getElementById('dict-dialog');
    if (dictDialog && btnNavDict) {
        const tabUser = document.getElementById('tab-user');
        const tabOfficial = document.getElementById('tab-official');
        const dictSearch = document.getElementById('dict-search');
        const dictTableContainer = document.getElementById('dict-table-container');
        const pagePrev = document.getElementById('page-prev');
        const pageNext = document.getElementById('page-next');
        const pageInfo = document.getElementById('page-info');

        let dictPage = 0;
        let dictPageSize = 10;
        let dictType = 'user';
        const dictUserControls = document.getElementById('dict-user-controls');

        btnNavDict.addEventListener('click', () => {
            dictPage = 0;
            dictDialog.showModal();
            loadDictionary();
            if (dictUserControls) dictUserControls.style.display = 'flex';
        });

        if (tabUser) tabUser.addEventListener('click', () => { dictType = 'user'; tabUser.classList.add('active'); if(tabOfficial) tabOfficial.classList.remove('active'); dictPage = 0; loadDictionary(); if (dictUserControls) dictUserControls.style.display = 'flex'; });
        if (tabOfficial) tabOfficial.addEventListener('click', () => { dictType = 'official'; tabOfficial.classList.add('active'); if(tabUser) tabUser.classList.remove('active'); dictPage = 0; loadDictionary(); if (dictUserControls) dictUserControls.style.display = 'none'; });
        if (dictSearch) dictSearch.addEventListener('input', () => { dictPage = 0; loadDictionary(); });
        if (pagePrev) pagePrev.addEventListener('click', () => { if (dictPage > 0) { dictPage--; loadDictionary(); } });
        if (pageNext) pageNext.addEventListener('click', () => { dictPage++; loadDictionary(); });

        async function loadDictionary() {
            try {
                const [items, totalPages] = await invoke('query_dictionary', {
                    dictType: dictType, page: dictPage, pageSize: dictPageSize, searchKey: dictSearch ? dictSearch.value.trim() : ''
                });

                if (pageInfo) {
                    const mask = currentLabels.label_page_info || '第 {} / {} 頁';
                    pageInfo.textContent = mask.replace('{}', dictPage + 1).replace('{}', totalPages || 1);
                }
                if (pagePrev) pagePrev.disabled = dictPage === 0;
                if (pageNext) pageNext.disabled = totalPages === 0 || dictPage + 1 >= totalPages;
 
                const colKey = currentLabels.glossary_key ? currentLabels.glossary_key.replace(':', '') : '原文 (Key)';
                const colVal = currentLabels.glossary_value ? currentLabels.glossary_value.replace(':', '') : '翻譯 (Value)';
                const colAct = currentLabels.glossary_col_actions || '操作';
                const emptyText = currentLabels.glossary_empty || '無結果';

                let html = `<table class="dict-table"><thead><tr><th>${colKey}</th><th>${colVal}</th><th>${colAct}</th></tr></thead><tbody>`;
                if (!items || items.length === 0) { html += `<tr><td colspan="3" style="text-align:center;">${emptyText}</td></tr>`; }
                else {
                    items.forEach(([k, v]) => {
                        const safeK = k.replace(/'/g, "&apos;").replace(/"/g, "&quot;");
                        html += `<tr>
                            <td>${k}</td>
                            <td><input type="text" value="${v}" id="dict-val-${safeK}" class="dict-input" style="width:100%; box-sizing:border-box; background:transparent; color:inherit; border:1px solid #555; padding:4px;"></td>
                            <td>
                                <button class="small-btn save-item" data-key="${safeK}" style="padding:4px 8px;">💾</button>
                                ${dictType === 'user' ? `<button class="small-btn delete-item" data-key="${safeK}" style="background-color:#aa1111; color:#fff; padding:4px 8px;">🗑</button>` : ''}
                            </td>
                        </tr>`;
                    });
                }
                html += '</tbody></table>';
                dictTableContainer.innerHTML = html;

                document.querySelectorAll('.save-item').forEach(b => b.addEventListener('click', async (e) => {
                    const key = e.currentTarget.getAttribute('data-key');
                    const val = document.getElementById(`dict-val-${key}`).value;
                    await invoke('edit_dictionary_item', { key: key, value: val, delete: false });
                    const mask = currentLabels.status_dict_item_updated || '📖 字典更新：{}';
                    appendLog(mask.replace('{}', key)); 
                    loadDictionary();
                }));
 
                document.querySelectorAll('.delete-item').forEach(b => b.addEventListener('click', async (e) => {
                    const key = e.currentTarget.getAttribute('data-key');
                    const confirmMask = currentLabels.status_dict_item_delete_confirm || '確定刪除條目 {} 嗎？';
                    if (confirm(confirmMask.replace('{}', key))) {
                        await invoke('edit_dictionary_item', { key: key, value: '', delete: true });
                        loadDictionary();
                    }
                }));
            } catch (e) { 
                const mask = currentLabels.status_dict_load_failed || '❌ 載入字典失敗: {}';
                appendLog(mask.replace('{}', currentLabels[e] || e)); 
            }
        }

        // [NEW] 綁定字典的 新增/取代 按鈕
        const btnDictAdd = document.getElementById('btn-dict-add');
        const btnDictReplace = document.getElementById('btn-dict-replace');
        const dictInputKey = document.getElementById('dict-input-key');
        const dictInputValue = document.getElementById('dict-input-value');

        if (btnDictAdd && dictInputKey && dictInputValue) {
            btnDictAdd.addEventListener('click', async () => {
                const k = dictInputKey.value.trim();
                const v = dictInputValue.value.trim();
                if (!k) return alert(currentLabels.status_dict_key_empty);
                try {
                    await invoke('edit_dictionary_item', { key: k, value: v, delete: false });
                    const mask = currentLabels.status_dict_add_success;
                    appendLog(mask.replace('{}', k).replace('{}', v));
                    dictInputKey.value = ''; dictInputValue.value = '';
                    loadDictionary();
                } catch (e) { 
                    const mask = currentLabels.status_dict_add_failed;
                    appendLog(mask.replace('{}', currentLabels[e] || e)); 
                }
            });
        }
        if (btnDictReplace && dictInputKey && dictInputValue) {
            btnDictReplace.addEventListener('click', async () => {
                const oldV = dictInputKey.value.trim();
                const newV = dictInputValue.value.trim();
                if (!oldV || !newV) return alert(currentLabels.status_dict_replace_empty);
                const confirmMask = currentLabels.status_dict_replace_confirm;
                if (confirm(confirmMask.replace('{}', oldV).replace('{}', newV))) {
                     try {
                         await invoke('edit_dictionary_item', { key: oldV, value: newV, delete: false });
                         const mask = currentLabels.status_dict_replace_sent;
                         appendLog(mask.replace('{}', oldV).replace('{}', newV));
                         dictInputKey.value = ''; dictInputValue.value = '';
                         loadDictionary();
                     } catch (e) { 
                         const mask = currentLabels.status_dict_replace_failed;
                         appendLog(mask.replace('{}', currentLabels[e] || e)); 
                     }
                }
            });
        }

        const btnDictClear = document.getElementById('btn-dict-clear');
        const btnDictImport = document.getElementById('btn-dict-import');
        const btnDictExport = document.getElementById('btn-dict-export');

        if (btnDictClear) {
            btnDictClear.addEventListener('click', async () => {
                if (dictType !== 'user') return;
                const title = currentLabels.glossary_clear_title || '確定清空全部？';
                if (confirm(title)) {
                    try {
                        await invoke('clear_user_dictionary');
                        const mask = currentLabels.status_dict_clear_success;
                        appendLog(mask);
                        loadDictionary();
                    } catch (e) { 
                        const errMask = currentLabels.status_dict_replace_failed; 
                        appendLog(errMask.replace('{}', currentLabels[e] || e)); 
                    }
                }
            });
        }

        if (btnDictImport) {
            btnDictImport.addEventListener('click', async () => {
                if (dictType !== 'user') return;
                try {
                    const path = await invoke('open_path_dialog', { diagType: 'file' });
                    if (path) {
                        await invoke('import_user_dictionary', { filePath: path });
                        const mask = currentLabels.status_dict_import_success;
                        appendLog(mask);
                        loadDictionary();
                    }
                } catch (e) { 
                    const errMask = currentLabels.status_dict_add_failed;
                    appendLog(errMask.replace('{}', currentLabels[e] || e)); 
                }
            });
        }

        if (btnDictExport) {
            btnDictExport.addEventListener('click', async () => {
                try {
                    const path = await invoke('open_path_dialog', { diagType: 'save_file' });
                    if (path) {
                        const p = path.endsWith('.json') ? path : path + '.json';
                        await invoke('export_user_dictionary', { filePath: p });
                        const mask = currentLabels.status_dict_export_success;
                        appendLog(mask.replace('{}', p));
                    }
                } catch (e) { 
                    const errMask = currentLabels.status_dict_replace_failed;
                    appendLog(errMask.replace('{}', currentLabels[e] || e)); 
                }
            });
        }
    }

    // --- 🌍 8. 前端 i18n 動態更新 ---


    // 🖱 全局：為數字輸入框綁定滾輪事件
    document.querySelectorAll('input[type="number"]').forEach(input => {
        input.addEventListener('wheel', (e) => {
            if (document.activeElement === input) {
                e.preventDefault();
                const step = parseFloat(input.step) || 1;
                const min = parseFloat(input.min) || -Infinity;
                const max = parseFloat(input.max) || Infinity;
                let val = parseFloat(input.value) || 0;
                val += e.deltaY < 0 ? step : -step;
                val = Math.max(min, Math.min(max, val));
                // 小數點處理以防浮點誤差
                const decimals = (input.step.split('.')[1] || '').length;
                input.value = val.toFixed(decimals);
                input.dispatchEvent(new Event('change'));
            }
        });
    });
});

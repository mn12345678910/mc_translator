const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener('DOMContentLoaded', async () => {
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
            langs.forEach(l => {
                const opt = document.createElement('option');
                opt.value = l;
                opt.textContent = l === 'zh_tw' ? '繁體中文 (zh_tw)' : l === 'en_us' ? 'English (en_us)' : l;
                uiLang.appendChild(opt);
            });
        } catch (e) {
            console.error('無法載入語言清單', e);
        }
    }
    await loadUiLangs();

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
        } catch (e) {
            appendLog(`❌ 載入配置失敗: ${e}`);
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

            currentConfig.system_prompt = systemPrompt.value;
            currentConfig.user_prompt = userPrompt.value;

            currentConfig.skip_json = chkSkipJson.checked;
            currentConfig.skip_js = chkSkipJs.checked;
            currentConfig.skip_jar = chkSkipJar.checked;
            currentConfig.skip_book = chkSkipBook.checked;
            currentConfig.enable_llm_log = chkLlmLog.checked;

            await invoke('save_config', { config: currentConfig });
            appendLog('✅ 核心參數儲存成功！');
            updateUiLanguage(); // [NEW] 儲存後也重新整理語言
        } catch (e) {
            appendLog(`❌ 儲存配置失敗: ${e}`);
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

            await invoke('save_style_config', { style: currentStyle });
            appendLog('🎨 調色盤與佈局保存成功！');
            applyColors(currentStyle);
            document.documentElement.style.setProperty('--font-size', fontSize.value + 'px');

        } catch (e) {
            appendLog(`❌ 保存樣式失敗: ${e}`);
        }
    }


    // --- 🎮 4. 面板熔斷鎖狀態機 ---
    function setRunningState(isRunning) {
        btnTranslate.disabled = isRunning;
        
        // 🔒 鎖定所有輸入框與面板以防誤觸 (100% 復刻熔斷閥)
        const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
        inputs.forEach(el => el.disabled = isRunning);
        
        if (isRunning) {
            btnPause.style.display = 'inline-block';
            btnStop.style.display = 'inline-block';
            btnPause.textContent = '⏸️ 暫停';
        } else {
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
            appendLog(`❌ 瀏覽路徑失敗: ${e}`);
        }
    }

    btnBrowseFile.addEventListener('click', () => browsePath('file', inputPath));
    btnBrowseDir.addEventListener('click', () => browsePath('dir', inputPath));
    btnBrowseOutput.addEventListener('click', () => browsePath('dir', outputDir)); // [NEW]

    // --- 其他邏輯 ---
    async function loadModels() {
        const provider = apiProvider.value;
        selectedModel.innerHTML = '<option value="">載入中...</option>';
        try {
            const models = await invoke('get_models_from_provider', { provider });
            selectedModel.innerHTML = '';
            models.forEach(m => {
                const opt = document.createElement('option');
                opt.value = m; opt.textContent = m;
                selectedModel.appendChild(opt);
            });
        } catch (e) {
            selectedModel.innerHTML = '<option value="">(無可用模型)</option>';
        }
    }

    function toggleOllamaGroup() {
        ollamaUrlGroup.style.display = apiProvider.value === 'Ollama' ? 'block' : 'none';
    }

    apiProvider.addEventListener('change', async () => { toggleOllamaGroup(); await loadModels(); });
    btnSaveConfig.addEventListener('click', saveConfig);
    btnSaveStyle.addEventListener('click', saveStyle);

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
            await invoke('save_style_config', { style: currentStyle });
            await loadStyle();
            appendLog(`🌓 主題已切換為：${currentStyle.theme === 'dark' ? '暗色' : '亮色'}，若未完全套用請重新整理。`);
        });
    }

    // --- 🚀 翻譯執行與監聽 ---
    btnTranslate.addEventListener('click', async () => {
        const path = inputPath.value.trim();
        if (!path) { appendLog('⚠️ 請輸入或選取待翻譯路徑！'); return; }
        setRunningState(true);
        logOutput.innerHTML = '';
        appendLog('🚀 翻譯任務開始發射...');

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
            appendLog('✅ 任務執行指令送達後端。');
        } catch (e) {
            appendLog(`❌ 執行出錯: ${e}`);
            setRunningState(false);
        }
    });

    btnPause.addEventListener('click', async () => {
        if (btnPause.textContent === '⏸️ 暫停') {
            await invoke('pause_translation'); btnPause.textContent = '▶️ 繼續運行'; appendLog('⏸️ 任務已暫停。面板解鎖，可改動設定。');
            const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
            inputs.forEach(el => el.disabled = false); // 暫停時解除鎖定
        } else {
            await invoke('resume_translation'); btnPause.textContent = '⏸️ 暫停'; appendLog('▶️ 任務已繼續。');
            const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
            inputs.forEach(el => el.disabled = true);
        }
    });

    btnStop.addEventListener('click', async () => { await invoke('stop_translation'); appendLog('⏹️ 正在送出終止信號...'); });

    // 📻 監聽後端 Event
    listen('translation-progress', (event) => {
        const { current, total, status } = event.payload;
        statusText.textContent = `${status} (${current} / ${total})`;
        progressBar.style.width = total > 0 ? `${(current / total) * 100}%` : '0%';
    });

    listen('translation-log', (event) => { appendLog(event.payload); });
    listen('translation-status', (event) => { statusText.textContent = event.payload; setRunningState(false); });

    function appendLog(text) {
        const p = document.createElement('p'); p.textContent = `[${new Date().toLocaleTimeString()}] ${text}`;
        logOutput.appendChild(p); logOutput.scrollTop = logOutput.scrollHeight;
    }

    function rgbToHex(arr) { if (!arr || arr.length < 3) return '#333333'; return '#' + arr.map(x => x.toString(16).padStart(2, '0')).join(''); }
    function hexToRgb(hex) { const bigint = parseInt(hex.slice(1), 16); return [(bigint >> 16) & 255, (bigint >> 8) & 255, bigint & 255]; }
    function applyColors(style) {
        const isDark = style.theme !== 'light';
        const bg = isDark ? style.dark_bg : style.light_bg;
        const txt = isDark ? style.dark_text : style.light_text;
        const btnBg = isDark ? style.dark_btn_bg : style.light_btn_bg;
        const btnTxt = isDark ? style.dark_btn_text : style.light_btn_text;

        if (bg) document.documentElement.style.setProperty('--bg-color', `rgb(${bg[0]},${bg[1]},${bg[2]})`);
        if (txt) document.documentElement.style.setProperty('--text-color', `rgb(${txt[0]},${txt[1]},${txt[2]})`);
        if (btnBg) document.documentElement.style.setProperty('--btn-bg', `rgb(${btnBg[0]},${btnBg[1]},${btnBg[2]})`);
        if (btnTxt) document.documentElement.style.setProperty('--btn-text', `rgb(${btnTxt[0]},${btnTxt[1]},${btnTxt[2]})`);

        // 套用圓角
        if (style.btn_rounding_enabled !== false) {
            document.documentElement.style.setProperty('--border-radius', `${style.btn_rounding_value ?? 4.0}px`);
        } else {
            document.documentElement.style.setProperty('--border-radius', '0px'); // 關閉圓角
        }

        // 套用進度條脈衝動畫
        if (style.progress_pulse_enabled) {
            const speed = style.progress_pulse_speed ?? 1.0;
            progressBar.style.animation = `pulse ${2.0 / Math.max(0.1, speed)}s infinite`;
        } else {
            progressBar.style.animation = 'none';
        }
    }

    function hexToColor(arr) { if (!arr || arr.length < 3) return '#fff'; return `rgb(${arr[0]},${arr[1]},${arr[2]})`; }

    // --- 🌍 6. 恢復預設參數與樣式 ---
    const btnRestoreConfig = document.getElementById('btn-restore-config');
    const btnRestoreStyle = document.getElementById('btn-restore-style');

    if (btnRestoreConfig) {
        btnRestoreConfig.addEventListener('click', async () => {
            if (!confirm('確定要將參數恢復為預設值嗎？')) return;
            try {
                const defaultConfig = await invoke('get_default_config');
                const currentPaths = {
                    api_key: apiKey.value, 
                    output_dir: document.getElementById('output-dir').value
                };
                currentConfig = { ...defaultConfig, ...currentPaths };
                await invoke('save_config', { config: currentConfig });
                appendLog('✅ 參數已恢復預設！');
                await loadConfig();
            } catch (e) { appendLog(`❌ 恢復參數失敗: ${e}`); }
        });
    }

    if (btnRestoreStyle) {
        btnRestoreStyle.addEventListener('click', async () => {
            if (!confirm('確定要將外觀佈景恢復為預設嗎？')) return;
            try {
                const defaultStyle = await invoke('get_default_style_config');
                currentStyle = defaultStyle;
                await invoke('save_style_config', { style: currentStyle });
                appendLog('🎨 外觀已恢復預設！');
                await loadStyle();
            } catch (e) { appendLog(`❌ 恢復樣式失敗: ${e}`); }
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

        btnNavDict.addEventListener('click', () => {
            dictPage = 0;
            dictDialog.showModal();
            loadDictionary();
        });

        if (tabUser) tabUser.addEventListener('click', () => { dictType = 'user'; tabUser.classList.add('active'); if(tabOfficial) tabOfficial.classList.remove('active'); dictPage = 0; loadDictionary(); });
        if (tabOfficial) tabOfficial.addEventListener('click', () => { dictType = 'official'; tabOfficial.classList.add('active'); if(tabUser) tabUser.classList.remove('active'); dictPage = 0; loadDictionary(); });
        if (dictSearch) dictSearch.addEventListener('input', () => { dictPage = 0; loadDictionary(); });
        if (pagePrev) pagePrev.addEventListener('click', () => { if (dictPage > 0) { dictPage--; loadDictionary(); } });
        if (pageNext) pageNext.addEventListener('click', () => { dictPage++; loadDictionary(); });

        async function loadDictionary() {
            try {
                const [items, totalPages] = await invoke('query_dictionary', {
                    dictType: dictType, page: dictPage, pageSize: dictPageSize, searchKey: dictSearch ? dictSearch.value.trim() : ''
                });

                if (pageInfo) pageInfo.textContent = `第 ${dictPage + 1} / ${totalPages || 1} 頁`;
                if (pagePrev) pagePrev.disabled = dictPage === 0;
                if (pageNext) pageNext.disabled = totalPages === 0 || dictPage + 1 >= totalPages;

                let html = '<table class="dict-table"><thead><tr><th>原文 (Key)</th><th>翻譯 (Value)</th><th>操作</th></tr></thead><tbody>';
                if (!items || items.length === 0) { html += '<tr><td colspan="3" style="text-align:center;">無結果</td></tr>'; }
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
                    appendLog(`📖 字典更新：${key}`); loadDictionary();
                }));

                document.querySelectorAll('.delete-item').forEach(b => b.addEventListener('click', async (e) => {
                    const key = e.currentTarget.getAttribute('data-key');
                    if (confirm(`確定刪除條目 ${key} 嗎？`)) {
                        await invoke('edit_dictionary_item', { key: key, value: '', delete: true });
                        loadDictionary();
                    }
                }));
            } catch (e) { appendLog(`❌ 載入字典失敗: ${e}`); }
        }
    }

    // --- 🌍 8. 前端 i18n 動態更新 ---
    async function updateUiLanguage() {
        try {
            const labels = await invoke('get_i18n_labels');
            const labelMap = {
                'btn-translate': labels.btn_run_trans,
                'btn-pause': labels.btn_pause,
                'btn-resume': labels.btn_resume,
                'btn-stop': labels.btn_stop,
                'btn-save-config': labels.btn_save_config || '💾 儲存核心參數',
                'btn-restore-config': labels.btn_restore_defaults,
                'btn-save-style': labels.btn_save_style || '💾 儲存佈景配置',
                'btn-restore-style': labels.btn_restore_defaults,
            };
            for (const [id, text] of Object.entries(labelMap)) {
                const el = document.getElementById(id);
                if (el && text) el.textContent = text;
            }
        } catch (e) { console.error('更新介面語言失敗', e); }
    }
    
    if (uiLang) {
        uiLang.addEventListener('change', async () => {
             await saveConfig();
             await updateUiLanguage();
        });
    }


    await loadConfig();
    await loadStyle();
});

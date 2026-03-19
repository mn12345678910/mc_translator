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
    const btnOpenDict = document.getElementById('btn-open-dict');

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
            apiKey.value = config.api_key || '';
            ollamaUrl.value = config.ollama_url || 'http://localhost:11434';
            batchSize.value = config.batch_size || 150;
            batchMaxChars.value = config.batch_max_chars || 3500;
            timeoutSec.value = config.timeout || 60;
            packFormat.value = config.pack_format ? config.pack_format.toString() : '15';
            uiLang.value = config.ui_lang || 'zh_tw';

            systemPrompt.value = config.system_prompt || '';
            userPrompt.value = config.user_prompt || '';

            chkSkipJson.checked = config.skip_json || false;
            chkSkipJs.checked = config.skip_js || false;
            chkSkipJar.checked = config.skip_jar || false;
            chkSkipBook.checked = config.skip_book || false;
            chkLlmLog.checked = config.enable_llm_log || false;

            toggleOllamaGroup();
            await loadModels();
            if (config.model) {
                selectedModel.value = config.model;
            }
        } catch (e) {
            appendLog(`❌ 載入配置失敗: ${e}`);
        }
    }

    async function saveConfig() {
        try {
            currentConfig.api_provider = apiProvider.value;
            currentConfig.api_key = apiKey.value;
            currentConfig.model = selectedModel.value;
            currentConfig.ollama_url = ollamaUrl.value;
            currentConfig.batch_size = parseInt(batchSize.value);
            currentConfig.batch_max_chars = parseInt(batchMaxChars.value);
            currentConfig.timeout = parseInt(timeoutSec.value);
            currentConfig.pack_format = parseInt(packFormat.value);
            currentConfig.ui_lang = uiLang.value;

            currentConfig.system_prompt = systemPrompt.value;
            currentConfig.user_prompt = userPrompt.value;

            currentConfig.skip_json = chkSkipJson.checked;
            currentConfig.skip_js = chkSkipJs.checked;
            currentConfig.skip_jar = chkSkipJar.checked;
            currentConfig.skip_book = chkSkipBook.checked;
            currentConfig.enable_llm_log = chkLlmLog.checked;

            await invoke('save_config', { config: currentConfig });
            appendLog('✅ 核心參數儲存成功！');
        } catch (e) {
            appendLog(`❌ 儲存配置失敗: ${e}`);
        }
    }

    // --- 🎨 3. 調色盤與字體縮放 ---
    async function loadStyle() {
        try {
            const style = await invoke('get_style_config');
            currentStyle = style;

            colorBg.value = rgbToHex(style.bg_color);
            colorText.value = rgbToHex(style.text_color);
            colorBtnBg.value = rgbToHex(style.btn_bg_color);
            colorBtnText.value = rgbToHex(style.btn_text_color);
            if (style.font_size) {
                fontSize.value = style.font_size;
                document.documentElement.style.setProperty('--font-size', style.font_size + 'px');
            }

            applyColors(style);
        } catch (e) {
            console.error(e);
        }
    }

    async function saveStyle() {
        try {
            currentStyle.bg_color = hexToRgb(colorBg.value);
            currentStyle.text_color = hexToRgb(colorText.value);
            currentStyle.btn_bg_color = hexToRgb(colorBtnBg.value);
            currentStyle.btn_text_color = hexToRgb(colorBtnText.value);
            currentStyle.font_size = parseFloat(fontSize.value);

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
            const path = await invoke('open_path_dialog', { dialogType: type });
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

    // --- 🚀 翻譯執行與監聽 ---
    btnTranslate.addEventListener('click', async () => {
        const path = inputPath.value.trim();
        if (!path) { appendLog('⚠️ 請輸入或選取待翻譯路徑！'); return; }
        setRunningState(true);
        logOutput.innerHTML = '';
        appendLog('🚀 翻譯任務開始發射...');

        try {
            await invoke('start_translation', {
                path,
                provider: apiProvider.value,
                model: selectedModel.value,
                apiKey: apiKey.value,
                ollamaUrl: ollamaUrl.value,
                batchSize: parseInt(batchSize.value),
                batchMaxChars: parseInt(batchMaxChars.value),
                timeout: parseInt(timeoutSec.value),
                userPrompt: userPrompt.value,
                systemPrompt: systemPrompt.value,
                packFormat: parseInt(packFormat.value)
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
        const rgb = style.bg_color;
        document.documentElement.style.setProperty('--bg-color', `rgb(${rgb[0]},${rgb[1]},${rgb[2]})`);
        document.documentElement.style.setProperty('--text-color', hexToColor(style.text_color));
        document.documentElement.style.setProperty('--btn-bg', hexToColor(style.btn_bg_color));
        document.documentElement.style.setProperty('--btn-text', hexToColor(style.btn_text_color));
    }
    function hexToColor(arr) { if (!arr || arr.length < 3) return '#fff'; return `rgb(${arr[0]},${arr[1]},${arr[2]})`; }

    await loadConfig();
    await loadStyle();
});

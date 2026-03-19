const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener('DOMContentLoaded', async () => {
    // 🛠️ 基礎控制元件
    const logOutput = document.getElementById('log-output');
    const progressBar = document.getElementById('progress-bar');
    const statusText = document.getElementById('status-text');
    const btnTranslate = document.getElementById('btn-translate');
    const btnPause = document.getElementById('btn-pause');
    const btnResume = document.getElementById('btn-resume');
    const btnStop = document.getElementById('btn-stop');
    const inputPath = document.getElementById('input-path');
    const btnBrowseFile = document.getElementById('btn-browse-file');
    const btnBrowseDir = document.getElementById('btn-browse-dir');

    // ⚙️ API 與常規參數面板
    const apiProvider = document.getElementById('api-provider');
    const apiKey = document.getElementById('api-key');
    const selectedModel = document.getElementById('selected-model');
    const ollamaUrl = document.getElementById('ollama-url');
    const ollamaUrlGroup = document.getElementById('ollama-url-group');
    const batchSize = document.getElementById('batch-size');
    const batchMaxChars = document.getElementById('batch-max-chars');
    const timeoutSec = document.getElementById('timeout-sec');
    const packFormat = document.getElementById('pack-format');
    const btnSaveConfig = document.getElementById('btn-save-config');

    // 🎨 調色盤管理元件
    const colorBg = document.getElementById('color-bg');
    const colorText = document.getElementById('color-text');
    const colorBtnBg = document.getElementById('color-btn-bg');
    const colorBtnText = document.getElementById('color-btn-text');
    const btnSaveStyle = document.getElementById('btn-save-style');

    // 🔧 開發者模式 Checkboxes
    const chkSkipJson = document.getElementById('chk-skip-json');
    const chkSkipJs = document.getElementById('chk-skip-js');
    const chkSkipJar = document.getElementById('chk-skip-jar');
    const chkSkipBook = document.getElementById('chk-skip-book');
    const chkLlmLog = document.getElementById('chk-llm-log');

    // 📖 字典管理器彈窗
    const btnOpenDict = document.getElementById('btn-open-dict');
    const dictDialog = document.getElementById('dict-dialog');
    const dictSearch = document.getElementById('dict-search');
    const tabUser = document.getElementById('tab-user');
    const tabOfficial = document.getElementById('tab-official');
    const dictTableContainer = document.getElementById('dict-table-container');
    const pagePrev = document.getElementById('page-prev');
    const pageNext = document.getElementById('page-next');
    const pageInfo = document.getElementById('page-info');

    // --- 全域狀態 ---
    let currentConfig = {};
    let currentStyle = {};
    let dictType = 'user'; // 'user' | 'official'
    let dictPage = 0;
    const dictPageSize = 25;
    let debounceTimer = null;

    const rgbToHex = ([r, g, b]) => "#" + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');
    const hexToRgb = (hex) => [
        parseInt(hex.slice(1, 3), 16),
        parseInt(hex.slice(3, 5), 16),
        parseInt(hex.slice(5, 7), 16)
    ];

    // 1. 載入並套用 Config 們
    try {
        currentStyle = await invoke('get_style_config');
        currentConfig = await invoke('get_config');

        // (A) 套用調色盤
        if (colorBg) colorBg.value = rgbToHex(currentStyle.dark_bg);
        if (colorText) colorText.value = rgbToHex(currentStyle.dark_text);
        if (colorBtnBg) colorBtnBg.value = rgbToHex(currentStyle.dark_btn_bg);
        if (colorBtnText) colorBtnText.value = rgbToHex(currentStyle.dark_btn_text);

        document.documentElement.style.setProperty('--bg-color', rgbToHex(currentStyle.dark_bg));
        document.documentElement.style.setProperty('--text-color', rgbToHex(currentStyle.dark_text));
        document.documentElement.style.setProperty('--btn-bg', rgbToHex(currentStyle.dark_btn_bg));
        document.documentElement.style.setProperty('--btn-text', rgbToHex(currentStyle.dark_btn_text));

        // (B) 套用 API 參數面板值
        if (apiProvider) apiProvider.value = currentConfig.api_provider;
        if (apiKey) apiKey.value = currentConfig.api_key;
        if (ollamaUrl) ollamaUrl.value = currentConfig.ollama_url;
        if (batchSize) batchSize.value = currentConfig.batch_size;
        if (batchMaxChars) batchMaxChars.value = currentConfig.batch_max_chars;
        if (timeoutSec) timeoutSec.value = currentConfig.timeout;
        if (packFormat) packFormat.value = currentConfig.pack_format;

        // 連動模型下拉或手動追加
        updateModelOptions(currentConfig.api_provider, currentConfig.model);
        toggleOllamaGroup(currentConfig.api_provider);

        // (C) 套用開發人員勾選
        if (chkSkipJson) chkSkipJson.checked = currentConfig.skip_json;
        if (chkSkipJs) chkSkipJs.checked = currentConfig.skip_js;
        if (chkSkipJar) chkSkipJar.checked = currentConfig.skip_jar;
        if (chkSkipBook) chkSkipBook.checked = currentConfig.skip_book;
        if (chkLlmLog) chkLlmLog.checked = currentConfig.enable_llm_log;

    } catch (err) { console.error("載入設定失敗:", err); }

    // --- 輔助函式組 ---
    function toggleOllamaGroup(provider) {
        if (ollamaUrlGroup) ollamaUrlGroup.style.display = (provider === 'Ollama') ? 'block' : 'none';
    }

    function updateModelOptions(provider, selected) {
        if (!selectedModel) return;
        selectedModel.innerHTML = ''; // 清空
        const models = provider === 'Gemini' 
            ? ['gemini-2.5-pro', 'gemini-2.5-flash', 'gemini-1.5-pro', 'gemini-1.5-flash']
            : ['llama3', 'mistral', 'qwen']; // 預設 Ollama 集份碼
        
        models.forEach(m => {
            const opt = document.createElement('option');
            opt.value = m; opt.textContent = m;
            if (m === selected) opt.selected = true;
            selectedModel.appendChild(opt);
        });
    }

    // 2. 監聽 Provider 切換
    if (apiProvider) {
        apiProvider.addEventListener('change', (e) => {
            toggleOllamaGroup(e.target.value);
            updateModelOptions(e.target.value, '');
        });
    }

    // 3. 💾 儲存核心 API 參數
    if (btnSaveConfig) {
        btnSaveConfig.addEventListener('click', async () => {
            currentConfig.api_provider = apiProvider.value;
            currentConfig.api_key = apiKey.value;
            currentConfig.model = selectedModel.value;
            currentConfig.ollama_url = ollamaUrl.value;
            currentConfig.batch_size = parseInt(batchSize.value);
            currentConfig.batch_max_chars = parseInt(batchMaxChars.value);
            currentConfig.timeout = parseInt(timeoutSec.value);
            currentConfig.pack_format = parseInt(packFormat.value);

            try {
                await invoke('save_config', { config: currentConfig });
                alert("核心設定已同步至 config.cfg！");
            } catch (err) { alert(`儲存設定失敗: ${err}`); }
        });
    }

    // 4. 📂 Native Dialog 選檔串接
    const attachDialog = (btn, type) => {
        if (btn) {
            btn.addEventListener('click', async () => {
                try {
                    const path = await invoke('open_path_dialog', { diagType: type });
                    if (path) inputPath.value = path;
                } catch (err) { alert(`開啟選取框失敗: ${err}`); }
            });
        }
    };
    attachDialog(btnBrowseFile, 'file');
    attachDialog(btnBrowseDir, 'dir');

    // 5. 監聽選色即時畫布預覽
    const attachLivePreview = (input, cssVar) => {
        if (input) input.addEventListener('input', e => document.documentElement.style.setProperty(cssVar, e.target.value));
    };
    attachLivePreview(colorBg, '--bg-color');
    attachLivePreview(colorText, '--text-color');
    attachLivePreview(colorBtnBg, '--btn-bg');
    attachLivePreview(colorBtnText, '--btn-text');

    // 6. 💾 儲存佈景設定
    if (btnSaveStyle) {
        btnSaveStyle.addEventListener('click', async () => {
            currentStyle.dark_bg = hexToRgb(colorBg.value);
            currentStyle.dark_text = hexToRgb(colorText.value);
            currentStyle.dark_btn_bg = hexToRgb(colorBtnBg.value);
            currentStyle.dark_btn_text = hexToRgb(colorBtnText.value);
            try {
                await invoke('save_style_config', { config: currentStyle });
                alert("佈景配置已同步！");
            } catch (err) { alert(`儲存佈景出錯: ${err}`); }
        });
    }

    // 7. 防抖式自動儲存 Config (開發人員過濾組)
    const triggerSaveConfigDebounced = () => {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(async () => {
            try {
                await invoke('save_config', { config: currentConfig });
                console.log("Config 自動保存落盤成功");
            } catch (err) { console.error("Config 自動保存出錯:", err); }
        }, 800);
    };

    const attachCheckboxSync = (el, field) => {
        if (el) el.addEventListener('change', (e) => {
            currentConfig[field] = e.target.checked;
            triggerSaveConfigDebounced();
        });
    };
    attachCheckboxSync(chkSkipJson, 'skip_json');
    attachCheckboxSync(chkSkipJs, 'skip_js');
    attachCheckboxSync(chkSkipJar, 'skip_jar');
    attachCheckboxSync(chkSkipBook, 'skip_book');
    attachCheckboxSync(chkLlmLog, 'enable_llm_log');

    // 8. 📖 字典管理器彈窗分頁運算 (完整保留)
    const renderDictionaryTable = (items) => {
        let html = `<table><thead><tr><th>原文 (Key)</th><th>譯文 (Value)</th><th style="width: 80px;">操作</th></tr></thead><tbody>`;
        items.forEach(([k, v]) => {
            html += `<tr>
                <td><strong>${k}</strong></td>
                <td><input type="text" class="dict-edit-input" data-key="${k}" value="${v}"></td>
                <td>
                    <button class="save-item-btn" data-key="${k}">💾</button>
                    ${dictType === 'user' ? `<button class="del-item-btn" data-key="${k}" style="background-color: #aa1111;">🗑️</button>` : ''}
                </td>
            </tr>`;
        });
        html += `</tbody></table>`;
        dictTableContainer.innerHTML = html;

        document.querySelectorAll('.save-item-btn').forEach(btn => {
            btn.addEventListener('click', async (e) => {
                const key = e.target.getAttribute('data-key');
                const input = document.querySelector(`.dict-edit-input[data-key="${key}"`);
                if (input) {
                    try {
                        await invoke('edit_dictionary_item', { key, value: input.value, delete: false });
                        alert(`「${key}」儲存成功！${dictType === 'official' ? '已自動轉存為使用者詞庫。' : ''}`);
                    } catch (err) { alert(`儲存失敗: ${err}`); }
                }
            });
        });

        document.querySelectorAll('.del-item-btn').forEach(btn => {
            btn.addEventListener('click', async (e) => {
                const key = e.target.getAttribute('data-key');
                if (confirm(`確定要刪除「${key}」嗎？`)) {
                    try {
                        await invoke('edit_dictionary_item', { key, value: '', delete: true });
                        loadDictionaryPage();
                    } catch (err) { alert(`刪除失敗: ${err}`); }
                }
            });
        });
    };

    const loadDictionaryPage = async () => {
        const searchKey = dictSearch ? dictSearch.value.trim() : "";
        try {
            const [items, totalPages] = await invoke('query_dictionary', {
                dictType, page: dictPage, pageSize: dictPageSize, searchKey
            });
            renderDictionaryTable(items);
            if (pageInfo) pageInfo.textContent = `第 ${dictPage + 1} / ${totalPages || 1} 頁`;
            if (pagePrev) pagePrev.disabled = (dictPage === 0);
            if (pageNext) pageNext.disabled = (dictPage + 1 >= (totalPages || 1));
        } catch (err) { console.error("加載字典失敗:", err); }
    };

    if (btnOpenDict) btnOpenDict.addEventListener('click', () => { if (dictDialog) { dictDialog.showModal(); loadDictionaryPage(); if (dictSearch) dictSearch.focus(); } });
    if (dictSearch) dictSearch.addEventListener('input', () => { dictPage = 0; loadDictionaryPage(); });

    const switchTab = (type, activeBtn, inactiveBtn) => {
        dictType = type; dictPage = 0;
        activeBtn.classList.add('active'); inactiveBtn.classList.remove('active');
        loadDictionaryPage();
    };
    if (tabUser) tabUser.addEventListener('click', () => switchTab('user', tabUser, tabOfficial));
    if (tabOfficial) tabOfficial.addEventListener('click', () => switchTab('official', tabOfficial, tabUser));

    if (pagePrev) pagePrev.addEventListener('click', () => { if (dictPage > 0) { dictPage--; loadDictionaryPage(); } });
    if (pageNext) pageNext.addEventListener('click', () => { dictPage++; loadDictionaryPage(); });

    // 9. 監聽日誌與進度事件
    listen('log_event', (e) => {
        const line = document.createElement('div');
        line.style.padding = "2px 0"; line.textContent = e.payload;
        logOutput.appendChild(line); logOutput.scrollTop = logOutput.scrollHeight;
    });

    listen('progress_event', (e) => {
        const [ratio, status] = e.payload;
        if (progressBar) progressBar.style.width = `${ratio * 100}%`;
        if (statusText) statusText.textContent = `進度: ${(ratio * 100).toFixed(1)}% - ${status}`;
    });

    // 10. 🎮 暫停 / 繼續 / 停止 按鈕連鎖發射器
    function setRunningState(isRunning, isPaused = false) {
        if (btnTranslate) btnTranslate.style.display = isRunning ? 'none' : 'inline-block';
        if (btnPause) btnPause.style.display = (isRunning && !isPaused) ? 'inline-block' : 'none';
        if (btnResume) btnResume.style.display = (isRunning && isPaused) ? 'inline-block' : 'none';
        if (btnStop) btnStop.style.display = isRunning ? 'inline-block' : 'none';
    }

    if (btnPause) btnPause.addEventListener('click', async () => { try { await invoke('pause_translation'); setRunningState(true, true); } catch (e) { console.error(e); } });
    if (btnResume) btnResume.addEventListener('click', async () => { try { await invoke('resume_translation'); setRunningState(true, false); } catch (e) { console.error(e); } });
    if (btnStop) btnStop.addEventListener('click', async () => { try { await invoke('stop_translation'); setRunningState(false); } catch (e) { console.error(e); } });

    // 11. 🚀 點擊翻譯 (參數流連鎖)
    if (btnTranslate) {
        btnTranslate.addEventListener('click', async () => {
            const path = inputPath ? inputPath.value.trim() : "";
            if (!path) { alert("請填寫待翻譯的輸入路徑！"); return; }
            if (logOutput) logOutput.innerHTML = '';
            if (statusText) statusText.textContent = "準備開始翻譯...";
            if (progressBar) progressBar.style.width = '0%';

            setRunningState(true);

            try {
                await invoke('start_translation', { inputPaths: [path], config: currentConfig });
                alert("翻譯任務執行結束！");
            } catch (err) { 
                alert(`翻譯出錯或已被中斷: ${err}`); 
            } finally {
                setRunningState(false);
            }
        });
    }
});

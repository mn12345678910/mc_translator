const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener('DOMContentLoaded', async () => {
    const logOutput = document.getElementById('log-output');
    const progressBar = document.getElementById('progress-bar');
    const statusText = document.getElementById('status-text');
    const btnTranslate = document.getElementById('btn-translate');
    const btnSave = document.getElementById('btn-save');
    const inputPath = document.getElementById('input-path');
    const apiProvider = document.getElementById('api-provider');

    // 1. 載入 Config 並初始化畫面上預設值
    try {
        const config = await invoke('get_config');
        console.log("載入設定:", config);
        
        if (apiProvider) {
            apiProvider.value = config.api_provider || "無";
        }
    } catch (err) {
        console.error("載入設定失敗:", err);
    }

    // 2. 監聽日誌事件
    listen('log_event', (event) => {
        const msg = event.payload;
        const line = document.createElement('div');
        line.style.padding = "2px 0";
        line.textContent = msg;
        logOutput.appendChild(line);
        logOutput.scrollTop = logOutput.scrollHeight; // 自動捲動
    });

    // 3. 監聽進度事件
    listen('progress_event', (event) => {
        const [ratio, status] = event.payload;
        if (progressBar) {
            progressBar.style.width = `${ratio * 100}%`;
        }
        if (statusText) {
            statusText.textContent = `進度: ${(ratio * 100).toFixed(1)}% - ${status}`;
        }
    });

    // 4. 點擊翻譯
    if (btnTranslate) {
        btnTranslate.addEventListener('click', async () => {
            const path = inputPath ? inputPath.value.trim() : "";
            if (!path) {
                alert("請填寫待翻譯的輸入路徑！");
                return;
            }

            if (logOutput) logOutput.innerHTML = ''; // 清除舊日誌
            if (statusText) statusText.textContent = "準備開始翻譯...";
            if (progressBar) progressBar.style.width = '0%';

            try {
                // 呼叫翻譯 Command
                await invoke('start_translation', { inputPaths: [path] });
                alert("翻譯任務執行結束！");
            } catch (err) {
                alert(`翻譯出錯: ${err}`);
            }
        });
    }

    // 5. 儲存設定 (輔助範例)
    if (btnSave) {
        btnSave.addEventListener('click', async () => {
            // 由於目前尚未實作雙向綁定表單，僅作連鎖測試
            alert("目前設定變更需由 Backend 推送至表單，此按鈕為概念驗證。");
        });
    }
});

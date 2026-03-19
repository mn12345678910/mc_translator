const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener('DOMContentLoaded', async () => {
    // 🛠️ 基礎控制元件
    const logOutput = document.getElementById('log-output');
    const progressBar = document.getElementById('progress-bar');
    const statusText = document.getElementById('status-text');
    const btnTranslate = document.getElementById('btn-translate');
    const inputPath = document.getElementById('input-path');

    // 🎨 調色盤管理元件
    const colorBg = document.getElementById('color-bg');
    const colorText = document.getElementById('color-text');
    const colorBtnBg = document.getElementById('color-btn-bg');
    const colorBtnText = document.getElementById('color-btn-text');
    const btnSaveStyle = document.getElementById('btn-save-style');

    // --- 輔助函式 ---
    const rgbToHex = ([r, g, b]) => "#" + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');
    const hexToRgb = (hex) => [
        parseInt(hex.slice(1, 3), 16),
        parseInt(hex.slice(3, 5), 16),
        parseInt(hex.slice(5, 7), 16)
    ];

    let currentStyle = {};

    // 1. 載入並套用 StyleConfig
    try {
        currentStyle = await invoke('get_style_config');
        console.log("載入樣式設定:", currentStyle);
        
        // 設定選色器對應預設值 (假設目前調整深色模式)
        if (colorBg) colorBg.value = rgbToHex(currentStyle.dark_bg);
        if (colorText) colorText.value = rgbToHex(currentStyle.dark_text);
        if (colorBtnBg) colorBtnBg.value = rgbToHex(currentStyle.dark_btn_bg);
        if (colorBtnText) colorBtnText.value = rgbToHex(currentStyle.dark_btn_text);

        // 即時初始套用至全 document
        document.documentElement.style.setProperty('--bg-color', rgbToHex(currentStyle.dark_bg));
        document.documentElement.style.setProperty('--text-color', rgbToHex(currentStyle.dark_text));
        document.documentElement.style.setProperty('--btn-bg', rgbToHex(currentStyle.dark_btn_bg));
        document.documentElement.style.setProperty('--btn-text', rgbToHex(currentStyle.dark_btn_text));

    } catch (err) {
        console.error("載入樣式失敗:", err);
    }

    // 2. 監聽選色器即時變動 (畫布預覽)
    const attachLivePreview = (input, cssVar) => {
        if (input) {
            input.addEventListener('input', (e) => {
                document.documentElement.style.setProperty(cssVar, e.target.value);
            });
        }
    };
    attachLivePreview(colorBg, '--bg-color');
    attachLivePreview(colorText, '--text-color');
    attachLivePreview(colorBtnBg, '--btn-bg');
    attachLivePreview(colorBtnText, '--btn-text');

    // 3. 儲存佈景設定
    if (btnSaveStyle) {
        btnSaveStyle.addEventListener('click', async () => {
            currentStyle.dark_bg = hexToRgb(colorBg.value);
            currentStyle.dark_text = hexToRgb(colorText.value);
            currentStyle.dark_btn_bg = hexToRgb(colorBtnBg.value);
            currentStyle.dark_btn_text = hexToRgb(colorBtnText.value);

            try {
                await invoke('save_style_config', { config: currentStyle });
                alert("佈景配置已同步至系統檔案！");
            } catch (err) {
                alert(`儲存佈景出錯: ${err}`);
            }
        });
    }

    // 4. 監聽核心翻譯管線日誌事件
    listen('log_event', (event) => {
        const msg = event.payload;
        const line = document.createElement('div');
        line.style.padding = "2px 0";
        line.textContent = msg;
        logOutput.appendChild(line);
        logOutput.scrollTop = logOutput.scrollHeight;
    });

    listen('progress_event', (event) => {
        const [ratio, status] = event.payload;
        if (progressBar) progressBar.style.width = `${ratio * 100}%`;
        if (statusText) statusText.textContent = `進度: ${(ratio * 100).toFixed(1)}% - ${status}`;
    });

    // 5. 點擊翻譯
    if (btnTranslate) {
        btnTranslate.addEventListener('click', async () => {
            const path = inputPath ? inputPath.value.trim() : "";
            if (!path) {
                alert("請填寫待翻譯的輸入路徑！");
                return;
            }
            if (logOutput) logOutput.innerHTML = ''; 
            if (statusText) statusText.textContent = "準備開始翻譯...";
            if (progressBar) progressBar.style.width = '0%';

            try {
                await invoke('start_translation', { inputPaths: [path] });
                alert("翻譯任務執行結束！");
            } catch (err) {
                alert(`翻譯出錯: ${err}`);
            }
        });
    }
});

import { loadConfig } from './modules/config.js';
import { loadUiLangs, updateUiLanguage } from './modules/i18n.js';
import { loadStyle } from './modules/style.js';
import { initDictionary, loadDictionary } from './modules/dictionary.js';

document.addEventListener('DOMContentLoaded', async () => {
    // 1. Standalone Startup Data load
    await loadConfig();
    await loadUiLangs();
    await loadStyle();
    await updateUiLanguage(); // 👈 載入初次介面語系

    // 2. Setup standard listener & trigger initial page
    initDictionary();
    await loadDictionary();

    // 3. Listen synchronize triggers
    if (window.__TAURI__) {
        window.__TAURI__.event.listen('dictionary-changed', () => {
            loadDictionary();
        });

        // 📢 監聽主視窗控制面板切換語言
        window.__TAURI__.event.listen('ui-lang-changed', async () => {
            await loadConfig(); // 重新載入最新 config
            await updateUiLanguage(); // 刷新字典標籤
            await loadDictionary(); // 重新讀取當前字典項
        });
    }
});

// frontend/modules/utils.js

export function debounce(fn, delay) {
    let timer = null;
    return function (...args) {
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, args), delay);
    };
}

export function rgbToHex(arr) {
    if (!arr || arr.length < 3) return '#333333';
    return '#' + arr.map((x) => x.toString(16).padStart(2, '0')).join('');
}

export function hexToRgb(hex) {
    const bigint = parseInt(hex.slice(1), 16);
    return [(bigint >> 16) & 255, (bigint >> 8) & 255, bigint & 255];
}

/**
 * 虛擬化日誌系統的進入點
 * 會將日誌內容轉送給全域的 __logViewer 實例
 */
export function appendLog(entry) {
    if (!window.__logViewer) {
        console.warn('Log viewer not yet initialized');
        return;
    }

    // 支援舊版字串輸入
    let data =
        typeof entry === 'string'
            ? {
                  level: 'Info',
                  message: entry,
                  timestamp: Date.now(),
              }
            : { ...entry };

    // 額外相容舊版：如果訊息包含錯誤標記（如 ❌）或 Error，自動升級為 Error 等級
    if (typeof entry === 'string' && (entry.includes('❌') || entry.includes('Error'))) {
        data.level = 'Error';
    }

    const level = (data.level || 'Info').toLowerCase();
    const timeStr = new Date(data.timestamp || Date.now()).toLocaleTimeString([], { hour12: false });

    window.__logViewer.appendLog(data.message, level, timeStr);
}

export function escapeHtml(str) {
    if (!str) return '';
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

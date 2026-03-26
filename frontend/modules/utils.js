// frontend/modules/utils.js
// 移除了未使用的 invoke 變數宣告

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

export function appendLog(entry) {
    const logOutput = document.getElementById('log-output');
    if (!logOutput) return;

    // 支援舊版字串輸入
    const data = typeof entry === 'string' ? { 
        level: 'Info', 
        message: entry, 
        timestamp: Date.now() 
    } : entry;

    const logLine = document.createElement('p');
    const timeStr = new Date(data.timestamp).toLocaleTimeString();
    logLine.textContent = `[${timeStr}] ${data.message}`;

    // 根據等級上色
    switch (data.level) {
        case 'Success':
            logLine.style.color = 'var(--success-color, #4caf50)';
            break;
        case 'Warn':
            logLine.style.color = 'var(--warning-color, #ff9800)';
            break;
        case 'Error':
            logLine.style.color = 'var(--danger-color, #ff6b6b)';
            break;
        default:
            // Info 使用預設文字顏色
            break;
    }

    logOutput.appendChild(logLine);
    logOutput.scrollTop = logOutput.scrollHeight;
    if (logOutput.childNodes.length > 501) {
        logOutput.removeChild(logOutput.firstChild);
    }
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

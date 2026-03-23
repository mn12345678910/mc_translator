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

export function appendLog(text) {
    const logOutput = document.getElementById('log-output');
    if (!logOutput) return;

    const logLine = document.createElement('p');
    logLine.textContent = `[${new Date().toLocaleTimeString()}] ${text}`;
    if (
        String(text).includes('❌') ||
        String(text).includes('⚠️') ||
        String(text).includes('Error') ||
        String(text).includes('⚠')
    ) {
        logLine.style.color = '#ff6b6b';
    }
    logOutput.appendChild(logLine);
    logOutput.scrollTop = logOutput.scrollHeight;
    if (logOutput.childNodes.length > 500) {
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

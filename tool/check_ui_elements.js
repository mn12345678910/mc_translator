const fs = require('fs');
const path = require('path');

const HTML_PATH = path.join(__dirname, '../frontend/index.html');

const REQUIRED_IDS = [
    'chk-debug-log',
    'label-debug-log',
    'chk-llm-log',
    'chk-debug-tools',
    'btn-nav-dev',
    'developer-settings',
];

function loadHtmlIds(html) {
    const ids = new Set();
    const idRegex = /id="([^"]+)"/g;
    let match = null;
    while ((match = idRegex.exec(html)) !== null) {
        ids.add(match[1]);
    }
    return ids;
}

function checkUiElements() {
    if (!fs.existsSync(HTML_PATH)) {
        console.error(`❌ 找不到 UI 入口檔案: ${HTML_PATH}`);
        process.exit(1);
    }

    const html = fs.readFileSync(HTML_PATH, 'utf8');
    const ids = loadHtmlIds(html);
    const missing = REQUIRED_IDS.filter((id) => !ids.has(id));

    if (missing.length > 0) {
        console.error('❌ UI 元件檢查失敗，以下 ID 遺失:');
        missing.forEach((id) => console.error(`  - ${id}`));
        process.exit(1);
    }

    console.log('✅ UI 元件檢查通過');
    process.exit(0);
}

checkUiElements();

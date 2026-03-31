const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.join(__dirname, '..');
const MODULES_DIR = path.join(ROOT_DIR, 'frontend/modules');
const RUST_COMMANDS_FILE = path.join(ROOT_DIR, 'src-tauri/src/commands.rs');
const MOCK_FILE = path.join(ROOT_DIR, 'tests/frontend/tauri_mock.js');
const MOCK_MODULE_JS = path.join(ROOT_DIR, 'frontend/modules/mock.js');
const INDEX_HTML = path.join(ROOT_DIR, 'frontend/index.html');
const MAIN_JS = path.join(ROOT_DIR, 'frontend/main.js');

/**
 * 遞歸掃描目錄下的檔案
 */
function getFilesRecursively(dir, fileList = []) {
    const files = fs.readdirSync(dir);
    files.forEach(file => {
        const filePath = path.join(dir, file);
        if (fs.statSync(filePath).isDirectory()) {
            if (file !== 'tests' && file !== 'node_modules') {
                getFilesRecursively(filePath, fileList);
            }
        } else if (file.endsWith('.js') || file.endsWith('.html')) {
            fileList.push(filePath);
        }
    });
    return fileList;
}

/**
 * 掃描前端目錄中的 Tauri API 調用
 */
function scanFrontendCalls() {
    const commands = new Set();
    const events = new Set();

    const frontendDir = path.join(ROOT_DIR, 'frontend');
    if (!fs.existsSync(frontendDir)) return { commands, events };

    const files = getFilesRecursively(frontendDir);
    for (const file of files) {
        const content = fs.readFileSync(file, 'utf8');

        // 搜尋 invoke('command')
        const invokeRegex = /invoke\(['"]([^'"]+)['"]/g;
        let match;
        while ((match = invokeRegex.exec(content)) !== null) {
            commands.add(match[1]);
        }

        // 搜尋 listen('event')
        const listenRegex = /listen\(['"]([^'"]+)['"]/g;
        while ((match = listenRegex.exec(content)) !== null) {
            events.add(match[1]);
        }
    }
    return { commands, events };
}

/**
 * 掃描 Rust 後端的指令實作
 */
function scanRustCommands() {
    const commands = new Set();
    if (!fs.existsSync(RUST_COMMANDS_FILE)) return commands;

    const content = fs.readFileSync(RUST_COMMANDS_FILE, 'utf8');
    // 搜尋 #[tauri::command] 後方的 pub fn 或 pub async fn
    // Regex: 匹配 #[tauri::command] 之後出現的 pub (async)? fn (\w+)
    const commandRegex = /#\[tauri::command\]\s+(?:#\[[^\]]+\]\s+)*pub\s+(?:async\s+)?fn\s+(\w+)/g;

    let match;
    while ((match = commandRegex.exec(content)) !== null) {
        commands.add(match[1]);
    }
    return commands;
}

/**
 * 驗證前後端同步狀況
 */
function verifySync() {
    console.log('🔍 正在執行前後端 Tauri API 介面對齊檢查...');

    const fe = scanFrontendCalls();
    const be = scanRustCommands();

    console.log(`- 前端請求指令: ${fe.commands.size} 個`);
    console.log(`- 後端實作指令: ${be.size} 個`);

    let hasError = false;

    // 1. 檢查前端是否有無效調用 (前端有但後端沒有)
    const invalidCalls = [];
    fe.commands.forEach(cmd => {
        if (!be.has(cmd)) {
            invalidCalls.push(cmd);
        }
    });

    if (invalidCalls.length > 0) {
        console.error(`❌ [同步錯誤] 前端調用了後端未定義的指令:`);
        invalidCalls.forEach(c => console.error(`   - ${c}`));
        hasError = true;
    }

    // 2. 驗證測試 Mock 環境 (靜態檢查)
    const mockContent = fs.existsSync(MOCK_FILE) ? fs.readFileSync(MOCK_FILE, 'utf8') : "";
    const moduleContent = fs.existsSync(MOCK_MODULE_JS) ? fs.readFileSync(MOCK_MODULE_JS, 'utf8') : "";
    const mainContent = fs.existsSync(MAIN_JS) ? fs.readFileSync(MAIN_JS, 'utf8') : "";
    const htmlContent = fs.existsSync(INDEX_HTML) ? fs.readFileSync(INDEX_HTML, 'utf8') : "";

    let mockMissing = [];
    fe.commands.forEach(cmd => {
        // 檢查測試用 mock、實例 Mock 模組、主程式 live mock 與 index.html 全域注入
        const inMock = mockContent.includes(cmd);
        const inModule = moduleContent.includes(`'${cmd}':`) || moduleContent.includes(`"${cmd}":`) || moduleContent.includes(`${cmd}:`);
        const inMain = mainContent.includes(`'${cmd}':`) || mainContent.includes(`"${cmd}":`) || mainContent.includes(`${cmd}:`);
        const inHtml = htmlContent.includes(`'${cmd}':`) || htmlContent.includes(`"${cmd}":`) || htmlContent.includes(`${cmd}:`);

        if (!inMock && !inModule && !inMain && !inHtml) {
            mockMissing.push(cmd);
        }
    });

    if (mockMissing.length > 0) {
        console.error(`❌ [同步錯誤] 以下 ${mockMissing.length} 個指令尚未在任何 Mock 環境中定義:`);
        mockMissing.forEach(c => console.error(`   - ${c}`));
        hasError = true;
    }

    if (!hasError) {
        console.log('✅ 前後端 Tauri 指令集對齊成功。');
        process.exit(0);
    } else {
        process.exit(1);
    }
}

try {
    verifySync();
} catch (err) {
    console.error('❌ 執行同步檢查時發生錯誤:', err.message);
    process.exit(1);
}

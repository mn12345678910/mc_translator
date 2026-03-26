const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.join(__dirname, '..');
const MODULES_DIR = path.join(ROOT_DIR, 'frontend/modules');
const MOCK_FILE = path.join(ROOT_DIR, 'tests/frontend/tauri_mock.js');

/**
 * 掃描前端模組中的 Tauri API 調用
 */
function scanFrontendCalls() {
    const commands = new Set();
    const events = new Set();
    
    if (!fs.existsSync(MODULES_DIR)) return { commands, events };

    const files = fs.readdirSync(MODULES_DIR).filter(f => f.endsWith('.js'));
    for (const file of files) {
        const content = fs.readFileSync(path.join(MODULES_DIR, file), 'utf8');
        
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
 * 驗證 tauri_mock.js 是否涵蓋了這些調用
 * 註：目前的 tauri_mock.js 使用動態代理，此腳本主要用於提醒開發者檢查測試案例中是否已實作對應 Mock
 */
function verifySync(calls) {
    console.log('🔍 正在掃描前端 Tauri API 調用...');
    console.log(`找到指令: ${Array.from(calls.commands).join(', ') || '無'}`);
    console.log(`找到事件: ${Array.from(calls.events).join(', ') || '無'}`);
    console.log('\n🚀 正在驗證測試 Mock 環境...');

    const mockContent = fs.readFileSync(MOCK_FILE, 'utf8');
    let warnings = 0;

    // 這裡我們檢查測試文件中是否至少「提到」了這些命令
    // 在進階版本中，我們可以檢查 tests/frontend/*.test.js 是否有應對的 mockInvoke.mockImplementation
    for (const cmd of calls.commands) {
        // 檢查指令是否在 Mock 文件或測試案例中被提及
        // 此處簡化處理，若未來需要更嚴格，可擴充掃描 tests/*.test.js
        if (!mockContent.includes(cmd)) {
            // 由於我們使用了動態代理，其實不一定要在 tauri_mock.js 寫死
            // 但為了「確保開發者記得更新測試」，我們可以建立一個清單文件或在 mock 中加入註釋
            console.warn(`[提醒] 指令 "${cmd}" 在前端被引用，請確保在對應的 .test.js 中有使用 mockInvoke 處理它。`);
            warnings++;
        }
    }

    console.log(`\n✅ 掃描完成。警告數: ${warnings}`);
    if (warnings > 0) {
        console.log('💡 提示：雖然動態代理能自動轉發調用，但請確保您的單元測試涵蓋了上述指令的邏輯。');
    }
}

try {
    const calls = scanFrontendCalls();
    verifySync(calls);
} catch (err) {
    console.error('❌ 執行同步檢查時發生錯誤:', err.message);
    process.exit(1);
}

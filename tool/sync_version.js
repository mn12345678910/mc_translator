const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.join(__dirname, '..');
const CARGO_PATH = path.join(ROOT_DIR, 'Cargo.toml');
const TAURI_PATH = path.join(ROOT_DIR, 'src-tauri/tauri.conf.json');

/**
 * 從 Cargo.toml 提取版本號
 */
function getCargoVersion() {
    const content = fs.readFileSync(CARGO_PATH, 'utf8');
    const match = content.match(/^version\s*=\s*["']([^"']+)["']/m);
    return match ? match[1] : null;
}

/**
 * 同步版本號至 tauri.conf.json
 */
function syncVersion() {
    const version = getCargoVersion();
    if (!version) {
        console.error('❌ 無法從 Cargo.toml 找到版本號。');
        process.exit(1);
    }

    if (!fs.existsSync(TAURI_PATH)) {
        console.warn('⚠️ 找不到 tauri.conf.json (跳過)');
        return;
    }

    const tauriContent = fs.readFileSync(TAURI_PATH, 'utf8');
    const tauriConfig = JSON.parse(tauriContent);

    if (tauriConfig.version !== version) {
        console.log(`🔄 同步版本號: ${tauriConfig.version} -> ${version} (以 Cargo.toml 為準)`);
        tauriConfig.version = version;

        // 保持格式美化
        fs.writeFileSync(TAURI_PATH, JSON.stringify(tauriConfig, null, 2) + '\n');

        // 返回 1 讓 pre-commit 報錯並要求重新 git add
        process.exit(1);
    } else {
        console.log(`✅ 版本號一致 (${version})`);
    }
}

try {
    syncVersion();
} catch (err) {
    console.error('❌ 版本同步失敗:', err.message);
    process.exit(1);
}

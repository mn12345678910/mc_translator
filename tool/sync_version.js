const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.join(__dirname, '..');
const CARGO_PATH = path.join(ROOT_DIR, 'Cargo.toml');
const TAURI_CONF_PATH = path.join(ROOT_DIR, 'src-tauri/tauri.conf.json');
const PACKAGE_JSON_PATH = path.join(ROOT_DIR, 'package.json');
const TAURI_CARGO_PATH = path.join(ROOT_DIR, 'src-tauri/Cargo.toml');

/**
 * 從 Cargo.toml 提取版本號
 */
function getCargoVersion() {
    const content = fs.readFileSync(CARGO_PATH, 'utf8');
    const match = content.match(/^version\s*=\s*["']([^"']+)["']/m);
    return match ? match[1] : null;
}

/**
 * 同步版本號至各個標籤檔案
 */
function syncVersion() {
    const version = getCargoVersion();
    if (!version) {
        console.error('❌ 無法從 Cargo.toml 找到版本號。');
        process.exit(1);
    }

    let hasChanged = false;

    // 1. 同步至 tauri.conf.json
    if (fs.existsSync(TAURI_CONF_PATH)) {
        const content = fs.readFileSync(TAURI_CONF_PATH, 'utf8');
        const json = JSON.parse(content);
        if (json.version !== version) {
            console.log(`🔄 同步 tauri.conf.json: ${json.version} -> ${version}`);
            json.version = version;
            fs.writeFileSync(TAURI_CONF_PATH, JSON.stringify(json, null, 2) + '\n');
            hasChanged = true;
        }
    }

    // 2. 同步至 package.json
    if (fs.existsSync(PACKAGE_JSON_PATH)) {
        const content = fs.readFileSync(PACKAGE_JSON_PATH, 'utf8');
        const json = JSON.parse(content);
        if (json.version !== version) {
            console.log(`🔄 同步 package.json: ${json.version} -> ${version}`);
            json.version = version;
            fs.writeFileSync(PACKAGE_JSON_PATH, JSON.stringify(json, null, 2) + '\n');
            hasChanged = true;
        }
    }

    // 3. 同步至 src-tauri/Cargo.toml
    if (fs.existsSync(TAURI_CARGO_PATH)) {
        const content = fs.readFileSync(TAURI_CARGO_PATH, 'utf8');
        const updatedContent = content.replace(/^version\s*=\s*["']([^"']+)["']/m, `version = "${version}"`);
        if (content !== updatedContent) {
            console.log(`🔄 同步 src-tauri/Cargo.toml -> ${version}`);
            fs.writeFileSync(TAURI_CARGO_PATH, updatedContent);
            hasChanged = true;
        }
    }

    if (hasChanged) {
        console.log('✅ 版本同步完成。請重新 git add 變更。');
        process.exit(1);
    } else {
        console.log(`✅ 所有檔案版本號一致 (${version})`);
    }
}

try {
    syncVersion();
} catch (err) {
    console.error('❌ 版本同步失敗:', err.message);
    process.exit(1);
}

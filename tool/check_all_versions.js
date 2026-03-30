const fs = require('fs');
const path = require('path');

/**
 * 驗證專案中所有版本號設定是否一致
 */
function checkAllVersions() {
    const rootDir = path.join(__dirname, '..');
    const files = [
        { path: 'Cargo.toml', type: 'toml' },
        { path: 'package.json', type: 'json' },
        { path: 'src-tauri/tauri.conf.json', type: 'json' },
        { path: 'src-tauri/Cargo.toml', type: 'toml' }
    ];

    const versions = {};

    for (const file of files) {
        const fullPath = path.join(rootDir, file.path);
        if (!fs.existsSync(fullPath)) {
            console.warn(`⚠️ 檔案未找到: ${file.path}`);
            continue;
        }

        const content = fs.readFileSync(fullPath, 'utf8');
        let version = null;

        if (file.type === 'json') {
            version = JSON.parse(content).version;
        } else if (file.type === 'toml') {
            const match = content.match(/^version\s*=\s*["']([^"']+)["']/m);
            version = match ? match[1] : null;
        }

        if (!version) {
            console.error(`❌ [版本號讀取失敗] ${file.path}: 無法提取版本號。`);
            process.exit(1);
        }

        versions[file.path] = version;
    }

    const versionList = Object.values(versions);
    const uniqueVersions = [...new Set(versionList)];

    if (uniqueVersions.length > 1) {
        console.error('\n❌ [版本號不一致] 檢測到專案內存多個不同版本號：');
        for (const [file, ver] of Object.entries(versions)) {
            console.error(`   - ${file}: ${ver}`);
        }
        console.error('\n👉 請執行 `node tool/sync_version.js` 來同步版本號並重新提交。\n');
        process.exit(1);
    }

    console.log(`✅ 版本號驗證通過: ${uniqueVersions[0]}`);
    process.exit(0);
}

checkAllVersions();

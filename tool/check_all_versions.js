const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

/**
 * 獲取 Git 歷史中的最新標籤
 */
function getLatestTag() {
    try {
        // 使用 git describe 獲取最新標籤，若無標籤則返回 null
        return execSync('git describe --tags --abbrev=0', { stdio: 'pipe' }).toString().trim();
    } catch (e) {
        return null;
    }
}

/**
 * 語義化版本比對 (x.y.z)
 * 返回: 1 (v1 > v2), -1 (v1 < v2), 0 (v1 == v2)
 */
function compareVersions(v1, v2) {
    const clean = (v) =>
        v
            .replace(/^v/, '')
            .split('.')
            .map((n) => parseInt(n, 10) || 0);
    const parts1 = clean(v1);
    const parts2 = clean(v2);

    for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
        const p1 = parts1[i] || 0;
        const p2 = parts2[i] || 0;
        if (p1 > p2) return 1;
        if (p1 < p2) return -1;
    }
    return 0;
}

/**
 * 驗證專案中所有版本號設定是否一致，且不落後於 Git 標籤
 */
function checkAllVersions() {
    // === 檢查是否為 Tag 推送 ===
    const isTagPush = (() => {
        try {
            const input = fs.readFileSync(0, 'utf8').trim();
            if (!input) return false;
            const lines = input.split('\n');
            for (const line of lines) {
                const [localRef] = line.split(' ');
                if (localRef && localRef.startsWith('refs/tags/')) {
                    return true;
                }
            }
            return false;
        } catch (e) {
            return false;
        }
    })();

    const rootDir = path.join(__dirname, '..');
    const files = [
        { path: 'Cargo.toml', type: 'toml' },
        { path: 'package.json', type: 'json' },
        { path: 'src-tauri/tauri.conf.json', type: 'json' },
        { path: 'src-tauri/Cargo.toml', type: 'toml' },
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
            try {
                version = JSON.parse(content).version;
            } catch (e) {
                console.error(`❌ [JSON 解析失敗] ${file.path}`);
            }
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

    // 1. 檢查檔案之間的一致性
    if (uniqueVersions.length > 1) {
        console.error('\n❌ [版本號不一致] 檢測到專案內容多個不同版本號：');
        for (const [file, ver] of Object.entries(versions)) {
            console.error(`   - ${file}: ${ver}`);
        }
        console.error('\n👉 請執行 `node tool/sync_version.js` 來同步版本號並重新提交。\n');
        process.exit(1);
    }

    const currentVersion = uniqueVersions[0];
    console.log(`✅ 檔案版本號一致: ${currentVersion}`);

    // 2. 只在 Tag 推送時檢查是否落後於 Git 已發佈標籤
    if (isTagPush) {
        const latestTag = getLatestTag();
        if (latestTag) {
            const compareResult = compareVersions(currentVersion, latestTag);

            if (compareResult < 0) {
                console.error(`\n❌ [版本號落後] 檔案版本 (${currentVersion}) 低於 Git 最新發佈標籤 (${latestTag})。`);
                console.error('💡 這是為了防止誤將過舊版本推送。請更新所有 manifest 檔案的版本號後再開啟 Commit。\n');
                process.exit(1);
            } else if (compareResult === 0) {
                console.log(`✅ 標籤同步驗證通過: 版本號等於最新標籤 (${latestTag})。`);
            } else {
                console.log(`✅ 標籤同步驗證通過: 版本號 (${currentVersion}) 高於最新標籤 (${latestTag})。`);
            }
        } else {
            console.log('ℹ️ 未發現任何 Git 標籤，跳過標籤對比。');
        }
    } else {
        console.log('ℹ️ 非 Tag 推送，跳過版本 vs 標籤檢查。');
    }

    process.exit(0);
}

checkAllVersions();

const fs = require('fs');
const path = require('path');

/**
 * 檢查指定目錄下的所有 JSON 檔案是否具備相同的 Key，且不含空字串。
 * @param {string} dirPath
 */
function checkI18nAlignment(dirPath) {
    console.log(`🔍 正在檢查目錄對齊情況: ${dirPath}`);

    if (!fs.existsSync(dirPath)) {
        console.warn(`⚠️ 目錄不存在: ${dirPath} (跳過)`);
        return true;
    }

    const files = fs.readdirSync(dirPath).filter(f => f.endsWith('.json'));
    if (files.length === 0) return true;

    // 1. 讀取所有檔案並提取所有出現過的 Key
    const masterKeys = new Set();
    const fileContents = {};

    files.forEach(file => {
        const fullPath = path.join(dirPath, file);
        try {
            const data = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
            fileContents[file] = data;
            Object.keys(data).forEach(key => masterKeys.add(key));
        } catch (e) {
            console.error(`❌ 無法讀取或解析 JSON: ${file}`, e);
            process.exit(1);
        }
    });

    // 2. 檢查每個檔案的缺失狀況與空值狀況
    let hasError = false;
    const sortedMasterKeys = Array.from(masterKeys).sort();

    files.forEach(file => {
        const data = fileContents[file];
        const missing = [];
        const untranslated = [];

        sortedMasterKeys.forEach(key => {
            if (!(key in data)) {
                missing.push(key);
            } else if (data[key] === "") {
                untranslated.push(key);
            }
        });

        if (missing.length > 0) {
            console.error(`❌ [${file}] 缺失以下 Key (${missing.length}):`);
            console.error(`   - ${missing.join(', ')}`);
            hasError = true;
        }

        if (untranslated.length > 0) {
            console.error(`❌ [${file}] 偵測到「空字串」(未翻譯) (${untranslated.length}):`);
            console.error(`   - ${untranslated.join(', ')}`);
            hasError = true;
        }
    });

    return !hasError;
}

// 執行檢查
const targetDirs = [
    path.join(__dirname, '../src/i18n_assets/gui'),
    path.join(__dirname, '../src/i18n_assets/cli')
];

let allSuccess = true;
targetDirs.forEach(dir => {
    if (!checkI18nAlignment(dir)) {
        allSuccess = false;
    }
});

if (allSuccess) {
    console.log("✅ I18n 檢查通過！所有語言檔案結構完全對齊且無空值。");
    process.exit(0);
} else {
    process.exit(1);
}

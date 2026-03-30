const fs = require('fs');
const path = require('path');

/**
 * 自動補全 I18n 缺失的 Key (以 en_us.json 為基準)
 * @param {string} dirPath
 */
function scaffoldI18n(dirPath) {
    console.log(`🛠️ 正在掃描 I18n 目錄: ${dirPath}`);

    if (!fs.existsSync(dirPath)) return false;

    const baseFile = 'en_us.json';
    const basePath = path.join(dirPath, baseFile);
    if (!fs.existsSync(basePath)) {
        console.warn(`⚠️ 找不到基準檔案: ${baseFile} (跳過)`);
        return false;
    }

    const baseData = JSON.parse(fs.readFileSync(basePath, 'utf8'));
    const baseKeys = Object.keys(baseData);

    const otherFiles = fs.readdirSync(dirPath).filter(f => f.endsWith('.json') && f !== baseFile);
    let modifiedAny = false;

    otherFiles.forEach(file => {
        const fullPath = path.join(dirPath, file);
        const data = JSON.parse(fs.readFileSync(fullPath, 'utf8'));
        const fileKeys = Object.keys(data);

        let modified = false;
        baseKeys.forEach(key => {
            if (!(key in data)) {
                console.log(`✨ [${file}] 自動補全 Key: "${key}"`);
                data[key] = ""; // 設為空字串，將觸發後續的「未翻譯」檢查
                modified = true;
            }
        });

        if (modified) {
            // 重新排序 Key 保持一致性
            const sortedData = {};
            // 我們可以使用 baseKeys 的順序，或者單純 ABC 排序
            // 這裡選擇遵循 baseKeys 的順序
            baseKeys.forEach(k => {
                sortedData[k] = data[k] !== undefined ? data[k] : "";
            });

            // 處理其餘可能存在的 Key (雖然理論上此時應齊全)
            Object.keys(data).forEach(k => {
                if (!(k in sortedData)) sortedData[k] = data[k];
            });

            fs.writeFileSync(fullPath, JSON.stringify(sortedData, null, 2) + '\n');
            modifiedAny = true;
        }
    });

    return modifiedAny;
}

const targetDirs = [
    path.join(__dirname, '../src/i18n_assets/gui'),
    path.join(__dirname, '../src/i18n_assets/cli')
];

let modifiedTotal = false;
targetDirs.forEach(dir => {
    if (scaffoldI18n(dir)) modifiedTotal = true;
});

if (modifiedTotal) {
    console.warn("\n⚠️ 偵測到 I18n Key 不對齊，已自動補入佔位符 (\"\")。");
    console.warn("💡 請填寫這些翻譯內容，否則 pre-commit 檢查將無法通過。");
    process.exit(1);
} else {
    console.log("✅ 所有 I18n 檔案結構已與 en_us.json 對齊。");
    process.exit(0);
}

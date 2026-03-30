const fs = require('fs');
const path = require('path');

const FRONTEND_DIR = path.join(__dirname, '../frontend');
const ASSET_EXTENSIONS = ['.png', '.jpg', '.jpeg', '.gif', '.svg', '.webp', '.ico', '.json', '.mp3', '.wav'];
const CODE_EXTENSIONS = ['.html', '.js', '.css'];

function getAllFiles(dir, allFiles = []) {
    const files = fs.readdirSync(dir);
    files.forEach(file => {
        const fullPath = path.join(dir, file);
        if (fs.statSync(fullPath).isDirectory()) {
            getAllFiles(fullPath, allFiles);
        } else {
            allFiles.push(fullPath);
        }
    });
    return allFiles;
}

function runCleanup() {
    console.log('🔍 正在開始資源無用偵測 (Point B)...');

    const allFiles = getAllFiles(FRONTEND_DIR);
    const assets = allFiles.filter(f => ASSET_EXTENSIONS.includes(path.extname(f).toLowerCase()));
    const codeFiles = allFiles.filter(f => CODE_EXTENSIONS.includes(path.extname(f).toLowerCase()));

    if (assets.length === 0) {
        console.log('✅ 未在 frontend/ 中發現任何媒體或 JSON 資源，無需清理。');
        process.exit(0);
    }

    const unreferenced = [];
    const codeContents = codeFiles.map(f => fs.readFileSync(f, 'utf8')).join('\n');

    assets.forEach(asset => {
        const basename = path.basename(asset);
        // 簡單搜尋：檢查檔案名稱是否出現在任何程式碼中
        if (!codeContents.includes(basename)) {
            unreferenced.push(asset);
        }
    });

    if (unreferenced.length > 0) {
        console.warn(`⚠️ 偵測到 ${unreferenced.length} 個無效資源：`);
        unreferenced.forEach(asset => {
            const relPath = path.relative(process.cwd(), asset);
            console.log(`   - ${relPath}`);
        });
        console.log('\n💡 建議執行以下指令進行清理：');
        console.log(`   Remove-Item ${unreferenced.map(f => `'${path.relative(process.cwd(), f)}'`).join(', ')}`);
        // 為了不干擾 pre-push，此處輸出警告但不強制報錯（除非使用者要求）
        // 但根據計畫，我們保持 pre-push 的一致性
        process.exit(0);
    } else {
        console.log(`✅ 恭喜！所有 ${assets.length} 個資源皆已被程式碼引用。`);
        process.exit(0);
    }
}

runCleanup();

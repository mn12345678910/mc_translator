const fs = require('fs');
const path = require('path');

// 紀錄校驗結果
let hasError = false;

function scanMarkdownFiles(dir, files = []) {
    const list = fs.readdirSync(dir);
    list.forEach(file => {
        const fullPath = path.join(dir, file);
        const stat = fs.statSync(fullPath);
        if (stat.isDirectory()) {
            if (file !== 'node_modules' && file !== '.git' && file !== 'target') {
                scanMarkdownFiles(fullPath, files);
            }
        } else if (file.endsWith('.md')) {
            files.push(fullPath);
        }
    });
    return files;
}

function checkLinksInFile(filePath) {
    const content = fs.readFileSync(filePath, 'utf8');
    const dir = path.dirname(filePath);

    // 正則 1: [text](link)
    const linkRegex = /\[.*?\]\((.*?)\)/g;
    let match;
    const links = [];

    while ((match = linkRegex.exec(content)) !== null) {
        links.push(match[1]);
    }

    links.forEach(link => {
        // 排除外部連結
        if (link.startsWith('http://') || link.startsWith('https://')) return;
        // 排除錨點
        if (link.startsWith('#')) return;

        let targetPath = '';
        if (link.startsWith('file:///')) {
            targetPath = decodeURIComponent(link.replace('file:///', ''));
            if (process.platform === 'win32' && !targetPath.includes(':') && targetPath.startsWith('/')) {
                targetPath = targetPath.substring(1);
            }
        } else if (link.startsWith('/')) {
            // 將 / 視為工作區根目錄
            targetPath = path.resolve(process.cwd(), link.substring(1));
        } else {

            // 相對路徑
            const cleanLink = link.split('#')[0].split('?')[0];
            if (!cleanLink) return;

            // 如果包含通配符 (*)，僅檢查目錄是否存在
            if (cleanLink.includes('*')) {
                const parts = cleanLink.split('*');
                const cleanPart = parts[0];
                if (cleanPart.startsWith('/')) {
                    targetPath = path.join(process.cwd(), cleanPart.substring(1));
                } else {
                    targetPath = path.resolve(dir, cleanPart.startsWith('./') ? cleanPart : './' + cleanPart);
                }
                if (fs.existsSync(targetPath)) return; // 目錄存在即可
            } else {

                targetPath = path.resolve(dir, cleanLink.startsWith('./') ? cleanLink : './' + cleanLink);
            }
        }



        if (!fs.existsSync(targetPath)) {
            console.error(`❌ [連結失效] 檔案: ${path.relative(process.cwd(), filePath)}`);
            console.error(`   - 連結: ${link}`);
            console.error(`   - 找不到目標: ${targetPath}`);
            hasError = true;
        }
    });
}

function run() {
    const workspaceRoot = process.cwd();
    console.log(`🔍 正在掃描工作區的所有 Markdown 連結...`);

    const mdFiles = scanMarkdownFiles(workspaceRoot);
    mdFiles.forEach(checkLinksInFile);

    if (hasError) {
        console.error(`\n🛑 連結校驗失敗！請修正上述損壞的 Markdown 連結。`);
        process.exit(1);
    } else {
        console.log(`✅ 連結校驗通過！共掃描了 ${mdFiles.length} 個 Markdown 檔案。`);
        process.exit(0);
    }
}

run();

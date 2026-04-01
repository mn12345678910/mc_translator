const fs = require('fs');
const path = require('path');

const srcDir = path.resolve(__dirname, '../langs/gui');
const destDir = path.resolve(__dirname, '../frontend/public/langs/gui');

if (!fs.existsSync(srcDir)) {
    console.error(`❌ Source directory not found: ${srcDir}`);
    process.exit(0); // 不要中斷建置，只是沒語系
}

if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
}

fs.readdirSync(srcDir).forEach(file => {
    if (file.endsWith('.json')) {
        fs.copyFileSync(path.join(srcDir, file), path.join(destDir, file));
        console.log(`✅ Synced: ${file} -> frontend/public/langs/gui/`);
    }
});

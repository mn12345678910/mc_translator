const fs = require('fs');

/**
 * 驗證 Git Commit Message 是否符合 Conventional Commits 規範
 * 格式: <type>(<scope>): <subject>
 */
function checkCommitMsg() {
    // pre-commit 會將 message 檔案的路徑作為第一個參數傳入
    const msgPath = process.argv[2];
    if (!msgPath) {
        console.error('❌ 找不到 Commit Message 檔案路徑。');
        process.exit(1);
    }

    const msg = fs.readFileSync(msgPath, 'utf8').trim();

    // 排除合併提交 (Merge branch...)
    if (msg.startsWith('Merge ')) {
        process.exit(0);
    }

    // 正則表達式: 支援帶括號的 scope 與可選的驚嘆號 (Breaking Change)
    const regex = /^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(?:\(.+\))?!?: .+/;

    if (!regex.test(msg)) {
        console.error('\n❌ [無效的提交格式] 您的提交訊息不符合 Conventional Commits 規範。');
        console.error('💡 正確格式範例:');
        console.error('   feat(i18n): 增加日語支援');
        console.error('   fix(gui): 修復按鈕對齊問題');
        console.error('   docs: 更新開發文件');
        console.error('\n可用類型 (Type): feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert');
        process.exit(1);
    }

    console.log('✅ Commit Message 格式驗證通過。');
    process.exit(0);
}

checkCommitMsg();

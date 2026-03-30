const fs = require('fs');
const { execSync } = require('child_process');

/**
 * 檢查即將推送的標籤是否指向目前的提交 (HEAD)
 * 供 pre-push 鉤子使用
 */
function checkTagSync() {
    try {
        // 從 stdin 讀取即將推送的內容 (格式: <local ref> <local sha> <remote ref> <remote sha>)
        const input = fs.readFileSync(0, 'utf8').trim();
        if (!input) {
            console.log('✅ 無事可做 (無推送內容)');
            process.exit(0);
        }

        const lines = input.split('\n');
        const headSha = execSync('git rev-parse HEAD').toString().trim();

        for (const line of lines) {
            const [localRef, localSha, remoteRef, remoteSha] = line.split(' ');

            // 檢查是否為標籤推送 (refs/tags/...)
            if (localRef.startsWith('refs/tags/')) {
                const tagName = localRef.replace('refs/tags/', '');

                if (localSha !== headSha) {
                    console.error(`\n❌ [標籤同步錯誤] 標籤 '${tagName}' 指向的提交 (${localSha.substring(0, 7)}) 不是目前分支的最新提交 (${headSha.substring(0, 7)})。`);
                    console.error('💡 這通常發生在您更新了代碼但忘了在打標籤前執行 Commit，或者是標籤打錯了地方。');
                    console.error('👉 請先 Commit 後重新打標籤，或使用 git tag -d 刪除舊標籤後重試。\n');
                    process.exit(1);
                }

                console.log(`✅ 驗證通過: 標籤 '${tagName}' 已與 HEAD 同步。`);
            }
        }

        process.exit(0);
    } catch (err) {
        // 若發生錯誤 (例如無 stdin)，可能是手動執行，此時不應該阻斷流程
        console.warn('⚠️ 標籤同步檢查跳過或發生錯誤:', err.message);
        process.exit(0);
    }
}

checkTagSync();

describe('Tauri 應用端到端冒煙測試', () => {
    it('應該正確啟動並顯示標題', async () => {
        // 取得視窗標題
        const title = await browser.getTitle();
        console.log('現有視窗標題為:', title);
        
        // 驗證標題是否包含 mc_translator
        expect(title.toLowerCase()).toContain('mc_translator');
    });

    it('應能尋找翻譯按紐，且按紐應存在', async () => {
        const btn = await $('#btn-translate');
        const exists = await btn.isExisting();
        expect(exists).toBe(true);
    });
});

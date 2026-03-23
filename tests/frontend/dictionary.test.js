import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

// Mock 外部模組防止交互副作用
vi.mock('../../frontend/modules/utils.js', () => ({
    appendLog: vi.fn(),
    escapeHtml: (str) => String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;')
}));

describe('dictionary.js 字典管理模組', () => {
    let mockInvoke;
    let dictionaryModule;
    let stateModule;

    beforeAll(async () => {
        // 1. Mock Tauri API
        mockInvoke = vi.fn();
        globalThis.window = {
            __TAURI__: {
                core: { invoke: mockInvoke }
            }
        };

        // 2. Mock 全域視窗函數 (confirm, alert)
        globalThis.confirm = vi.fn();
        globalThis.alert = vi.fn();

        // 3. 動態載入
        dictionaryModule = await import('../../frontend/modules/dictionary.js');
        stateModule = await import('../../frontend/modules/state.js');
    });

    beforeEach(() => {
        // 模擬完整的 DOM 結構
        document.body.innerHTML = `
            <button id="btn-nav-dict"></button>
            <dialog id="dict-dialog">
                <button id="tab-user" class="active"></button>
                <button id="tab-official"></button>
                <input id="dict-search" />
                <div id="dict-user-controls" style="display: flex;">
                    <input id="dict-input-key" />
                    <input id="dict-input-value" />
                    <button id="btn-dict-add"></button>
                    <button id="btn-dict-replace"></button>
                    <button id="btn-dict-clear"></button>
                    <button id="btn-dict-import"></button>
                    <button id="btn-dict-export"></button>
                </div>
                <div id="dict-table-container"></div>
                <div id="dict-pagination">
                    <button id="page-prev" disabled></button>
                    <span id="page-info"></span>
                    <button id="page-next" disabled></button>
                </div>
            </dialog>
        `;

        // 讓 <dialog> 支援 showModal 模擬
        const dialog = document.getElementById('dict-dialog');
        dialog.showModal = vi.fn(() => dialog.setAttribute('open', ''));
        dialog.close = vi.fn(() => dialog.removeAttribute('open'));

        // 重設 Mock
        mockInvoke.mockReset();
        globalThis.confirm.mockReset();
        globalThis.alert.mockReset();
        vi.clearAllMocks();

        // 重設 State
        stateModule.state.currentLabels = {
            label_page_info: '第 {} / {} 頁',
            glossary_key: '原文:',
            glossary_value: '翻譯:',
            status_dict_item_updated: '📖 更新 {}',
            status_dict_add_success: '✅ 新增 {} -> {}'
        };
    });

    it('loadDictionary 應該獲取字典條目並渲染成 HTML 表格', async () => {
        // 模擬 Query 回傳：[ items, totalPages ]
        mockInvoke.mockResolvedValue([
            [['apple', '蘋果'], ['banana', '香蕉']], 
            2 // 總頁數
        ]);

        await dictionaryModule.loadDictionary();

        const container = document.getElementById('dict-table-container');
        expect(container.innerHTML).toContain('apple');
        expect(container.innerHTML).toContain('banana');
        expect(container.innerHTML).toContain('蘋果');

        // 驗證分頁資訊
        const pageInfo = document.getElementById('page-info');
        expect(pageInfo.textContent).toBe('第 1 / 2 頁');
        expect(document.getElementById('page-next').disabled).toBe(false);
    });

    it('點擊表格內的儲存按鈕應該觸發 edit_dictionary_item', async () => {
        mockInvoke.mockResolvedValue([[['apple', '蘋果']], 1]);

        await dictionaryModule.loadDictionary();

        const container = document.getElementById('dict-table-container');
        const input = container.querySelector('.dict-input');
        input.value = '青蘋果'; // 使用者修改數值

        const saveBtn = container.querySelector('.save-item');
        await saveBtn.dispatchEvent(new Event('click'));

        // 驗證 invoke 被呼叫
        expect(mockInvoke).toHaveBeenCalledWith('edit_dictionary_item', {
            key: 'apple',
            value: '青蘋果',
            delete: false
        });
    });

    it('initDictionary 應該正確設定新增條目 Event Listener', async () => {
        dictionaryModule.initDictionary();

        document.getElementById('dict-input-key').value = 'cat';
        document.getElementById('dict-input-value').value = '貓咪';

        const addBtn = document.getElementById('btn-dict-add');
        await addBtn.dispatchEvent(new Event('click'));

        expect(mockInvoke).toHaveBeenCalledWith('edit_dictionary_item', {
            key: 'cat',
            value: '貓咪',
            delete: false
        });
    });

    it('切換 Tab 應異動面板顯示狀態', async () => {
        // 模擬切換 Tab 時加載字典不報錯
        mockInvoke.mockResolvedValue([[], 1]);
        stateModule.state.currentLabels.status_dict_load_failed = '讀取失敗 {}';

        dictionaryModule.initDictionary();

        const tabOfficial = document.getElementById('tab-official');
        const controls = document.getElementById('dict-user-controls');

        // 切換至官方
        await tabOfficial.dispatchEvent(new Event('click'));
        expect(controls.style.display).toBe('none');

        // 切換回使用者
        const tabUser = document.getElementById('tab-user');
        await tabUser.dispatchEvent(new Event('click'));
        expect(controls.style.display).toBe('flex');
    });
    it('點擊表格內的刪除按鈕，點按確認後應調用刪除 API', async () => {
        stateModule.state.currentLabels.status_dict_item_delete_confirm = '刪除 {}?';
        mockInvoke.mockResolvedValue([[['apple', '蘋果']], 1]);
        globalThis.confirm.mockReturnValue(true); 

        await dictionaryModule.loadDictionary();

        const container = document.getElementById('dict-table-container');
        const deleteBtn = container.querySelector('.delete-item');
        
        await deleteBtn.dispatchEvent(new Event('click'));

        expect(mockInvoke).toHaveBeenCalledWith('edit_dictionary_item', {
            key: 'apple',
            value: '',
            delete: true
        });
    });

    it('新增條目時如果 Key 為空應觸發 alert 警告', async () => {
        stateModule.state.currentLabels.status_dict_key_empty = 'Key cannot be empty';
        dictionaryModule.initDictionary();

        document.getElementById('dict-input-key').value = ''; 
        document.getElementById('dict-input-value').value = '貓咪';

        const addBtn = document.getElementById('btn-dict-add');
        await addBtn.dispatchEvent(new Event('click'));

        expect(globalThis.alert).toHaveBeenCalled();
    });

    it('點擊匯入按鈕應開啟路徑選擇並觸發 import', async () => {
        mockInvoke.mockResolvedValue('C:/test.json'); 
        dictionaryModule.initDictionary();

        const importBtn = document.getElementById('btn-dict-import');
        await importBtn.dispatchEvent(new Event('click'));

        expect(mockInvoke).toHaveBeenCalledWith('open_path_dialog', { diagType: 'file' });
        expect(mockInvoke).toHaveBeenCalledWith('import_user_dictionary', { filePath: 'C:/test.json' });
    });
});


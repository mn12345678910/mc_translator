import { describe, it, expect, beforeEach, beforeAll, vi } from 'vitest';

// Mock 外部模組防止交互副作用
vi.mock('../../frontend/modules/utils.js', () => ({
    appendLog: vi.fn(),
    escapeHtml: (str) => String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;')
}));

import { appendLog } from '../../frontend/modules/utils.js';


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
            status_dict_add_success: '✅ 新增 {} -> {}',
            status_dict_replace_confirm: '替換 {} -> {}?',
            status_dict_replace_empty: '替換不可為空',
            status_dict_replace_failed: '替換失敗 {}',
            glossary_clear_title: '確定清除?',
            status_dict_clear_success: '已清除',
            status_dict_export_success: '已導出 {}',
            status_open_path_failed: '打開失敗 {}',
            status_dict_add_failed: '新增失敗 {}',
            status_dict_load_failed: '讀取失敗 {}'
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
        await input.dispatchEvent(new Event('change'));

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
        expect(controls.classList.contains('hidden')).toBe(true);

        // 切換回使用者
        const tabUser = document.getElementById('tab-user');
        await tabUser.dispatchEvent(new Event('click'));
        expect(controls.classList.contains('hidden')).toBe(false);
    });
    it('表格不應顯示操作欄位或刪除按鈕', async () => {
        mockInvoke.mockResolvedValue([[['apple', '蘋果']], 1]);

        await dictionaryModule.loadDictionary();

        const container = document.getElementById('dict-table-container');
        const deleteBtn = container.querySelector('.delete-item');

        expect(deleteBtn).toBeNull();
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

    describe('更多互動與邊界測試 (Extended Interactions)', () => {
        it('點擊替換按鈕應觸發替換 confirm 與 edit_dictionary_item', async () => {
            stateModule.state.currentLabels.status_dict_replace_confirm = '替換 {} -> {}?';
            globalThis.confirm.mockReturnValue(true);

            dictionaryModule.initDictionary();

            document.getElementById('dict-input-key').value = 'apple';
            document.getElementById('dict-input-value').value = '青蘋果';

            const replaceBtn = document.getElementById('btn-dict-replace');
            await replaceBtn.dispatchEvent(new Event('click'));

            expect(mockInvoke).toHaveBeenCalledWith('edit_dictionary_item', {
                key: 'apple',
                value: '青蘋果',
                delete: false
            });
        });

        it('點擊清除按鈕應觸發 clear_user_dictionary', async () => {
            stateModule.state.currentLabels.glossary_clear_title = '確定清除?';
            globalThis.confirm.mockReturnValue(true);

            dictionaryModule.initDictionary();

            const clearBtn = document.getElementById('btn-dict-clear');
            await clearBtn.dispatchEvent(new Event('click'));

            expect(mockInvoke).toHaveBeenCalledWith('clear_user_dictionary');
        });

        it('點擊導出按鈕應開啟儲存對話框並觸發 export', async () => {
            mockInvoke.mockResolvedValue('C:/export.json');

            dictionaryModule.initDictionary();

            const exportBtn = document.getElementById('btn-dict-export');
            await exportBtn.dispatchEvent(new Event('click'));

            expect(mockInvoke).toHaveBeenCalledWith('open_path_dialog', { diagType: 'save_file' });
            expect(mockInvoke).toHaveBeenCalledWith('export_user_dictionary', { filePath: 'C:/export.json' });
        });

        it('搜尋框輸入應重置頁碼並加載字典', async () => {
            mockInvoke.mockResolvedValue([[], 1]);

            dictionaryModule.initDictionary();

            const searchInput = document.getElementById('dict-search');
            searchInput.value = 'test';
            await searchInput.dispatchEvent(new Event('input'));

            expect(mockInvoke).toHaveBeenCalledWith('query_dictionary', expect.objectContaining({
                searchKey: 'test',
                page: 0
            }));
        });

        it('點擊分頁上一頁應遞減頁碼並加載字典', async () => {
             // 假設當前在第二頁 (page 1)
             // 為了設定 page，我們先模擬 next 的點擊
             mockInvoke.mockResolvedValue([[['A', 'B']], 3]);

             await dictionaryModule.loadDictionary();

             dictionaryModule.initDictionary();
             const nextBtn = document.getElementById('page-next');
             await nextBtn.dispatchEvent(new Event('click')); // 點擊下一頁，此時 page = 1

             const prevBtn = document.getElementById('page-prev');
             // 模擬 page-prev 的 EventListener 運作
             await prevBtn.dispatchEvent(new Event('click'));

             expect(mockInvoke).toHaveBeenLastCalledWith('query_dictionary', expect.objectContaining({
                  page: 0
             }));
        });

        it('點擊開啟 JSON 位置按鈕應觸發 open_dictionary_location', async () => {
            document.body.innerHTML += `<button id="btn-dict-open-json"></button>`;
            dictionaryModule.initDictionary();

            const openBtn = document.getElementById('btn-dict-open-json');
            await openBtn.dispatchEvent(new Event('click'));

            expect(mockInvoke).toHaveBeenCalledWith('open_dictionary_location', expect.objectContaining({
                dictType: 'user'
            }));
        });

        it('匯入拋出異常時應捕獲異常並讀取錯誤狀態', async () => {
             mockInvoke.mockImplementation(async (cmd) => {
                  if (cmd === 'open_path_dialog') return 'C:/some.json';
                  if (cmd === 'import_user_dictionary') throw 'import-failed';
                  if (cmd === 'query_dictionary') return [[], 1]; // 避免 loadDictionary 拋出 TypeError
                  return null;
             });

             dictionaryModule.initDictionary();

             const tabUser = document.getElementById('tab-user');
             await tabUser.dispatchEvent(new Event('click'));

             const importBtn = document.getElementById('btn-dict-import');
             await importBtn.dispatchEvent(new Event('click'));

             await vi.waitFor(() => {
                 expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('import-failed'));
             });
        });

        it('導出拋出異常時應捕獲異常並讀取錯誤狀態', async () => {
             mockInvoke.mockImplementation(async (cmd) => {
                  if (cmd === 'open_path_dialog') return 'C:/export.json';
                  if (cmd === 'export_user_dictionary') throw 'export-failed';
                  if (cmd === 'query_dictionary') return [[], 1]; // 預留 fallback
                  return null;
             });

             dictionaryModule.initDictionary();

             const exportBtn = document.getElementById('btn-dict-export');
             await exportBtn.dispatchEvent(new Event('click'));

             await vi.waitFor(() => {
                 expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('export-failed'));
             });
        });

        it('優先級開關切換時應重置頁碼並加載字典', async () => {
            document.body.innerHTML += `<input id="chk-glossary-priority" type="checkbox" />`;
            mockInvoke.mockResolvedValue([[], 1]);

            dictionaryModule.initDictionary();

            const chk = document.getElementById('chk-glossary-priority');
            chk.checked = true;
            await chk.dispatchEvent(new Event('change'));

            expect(mockInvoke).toHaveBeenCalledWith('query_dictionary', expect.objectContaining({
                 page: 0
            }));
        });

        it('新增條目拋出異常時應捕獲異常並讀取錯誤狀態', async () => {
             mockInvoke.mockImplementation(async (cmd) => {
                  if (cmd === 'edit_dictionary_item') throw 'add-failed';
                  if (cmd === 'query_dictionary') return [[], 1];
                  return null;
             });
             dictionaryModule.initDictionary();

             document.getElementById('dict-input-key').value = 'cat';
             await document.getElementById('btn-dict-add').dispatchEvent(new Event('click'));

             await vi.waitFor(() => {
                  expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('add-failed'));
             });
        });

        it('清除字典拋出異常時應捕獲異常並讀取錯誤狀態', async () => {
             mockInvoke.mockImplementation(async (cmd) => {
                  if (cmd === 'clear_user_dictionary') throw 'clear-failed';
                  if (cmd === 'query_dictionary') return [[], 1];
                  return null;
             });
             globalThis.confirm.mockReturnValue(true);
             dictionaryModule.initDictionary();

             await document.getElementById('btn-dict-clear').dispatchEvent(new Event('click'));

             await vi.waitFor(() => {
                  expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('clear-failed'));
             });
        });

        it('開啟 JSON 位置拋出異常時應捕獲異常並讀取錯誤狀態', async () => {
             mockInvoke.mockImplementation(async (cmd) => {
                  if (cmd === 'open_dictionary_location') throw 'open-failed';
                  if (cmd === 'query_dictionary') return [[], 1];
                  return null;
             });
             document.body.innerHTML += `<button id="btn-dict-open-json"></button>`;
             dictionaryModule.initDictionary();

             await document.getElementById('btn-dict-open-json').dispatchEvent(new Event('click'));

             await vi.waitFor(() => {
                  expect(appendLog).toHaveBeenCalledWith(expect.stringContaining('open-failed'));
             });
        });

        it('當 label_page_info 格式不符合預期應採取 fallback 行為', async () => {
            stateModule.state.currentLabels.label_page_info = '第 {} 頁';
            mockInvoke.mockResolvedValue([[['apple', '蘋果']], 2]);

            await dictionaryModule.loadDictionary();

            const pageInfo = document.getElementById('page-info');
            expect(pageInfo.textContent).toBe('第 1 頁');
        });

        it('點擊導航欄字典按鈕應開啟 Dialog 並加載字典', async () => {
            mockInvoke.mockResolvedValue([[], 1]);
            dictionaryModule.initDictionary();

            const btnNavDict = document.getElementById('btn-nav-dict');
            const dictDialog = document.getElementById('dict-dialog');

            await btnNavDict.dispatchEvent(new Event('click'));

            expect(mockInvoke).toHaveBeenCalledWith('open_dict_window');
        });
    });
});

// frontend/modules/dictionary.js
import { state } from './state.js';
import { appendLog, escapeHtml } from './utils.js';
import { updateToggleStateLabel } from './i18n.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };

let dictPage = 0;
let dictPageSize = 10;
let dictType = 'user';

export async function loadDictionary() {
    const dictSearch = document.getElementById('dict-search');
    const pageInfo = document.getElementById('page-info');
    const pagePrev = document.getElementById('page-prev');
    const pageNext = document.getElementById('page-next');
    const dictTableContainer = document.getElementById('dict-table-container');

    try {
        const [items, totalPages] = await invoke('query_dictionary', {
            dictType: dictType,
            page: dictPage,
            pageSize: dictPageSize,
            searchKey: dictSearch ? dictSearch.value.trim() : '',
        });

        if (pageInfo) {
            const mask = state.currentLabels.label_page_info || '第 {} / {} 頁';
            const parts = mask.split('{}');
            if (parts.length >= 3) {
                pageInfo.textContent = `${parts[0]}${dictPage + 1}${parts[1]}${totalPages || 1}${parts[2]}`;
            } else {
                // 退回機制
                pageInfo.textContent = mask.replace('{}', dictPage + 1).replace('{}', totalPages || 1);
            }
        }
        if (pagePrev) pagePrev.disabled = dictPage === 0;
        if (pageNext) pageNext.disabled = totalPages === 0 || dictPage + 1 >= totalPages;

        const colKey = (state.currentLabels.glossary_key || '原文').replace(':', '');
        const colVal = (state.currentLabels.glossary_value || '翻譯').replace(':', '');
        const colAct = state.currentLabels.glossary_col_actions || '操作';
        const emptyText = state.currentLabels.glossary_empty || '無資料';

        let html = `<table class="dict-table"><thead><tr><th>${colKey}</th><th>${colVal}</th><th>${colAct}</th></tr></thead><tbody>`;
        if (!items || items.length === 0) {
            html += `<tr><td colspan="3" style="text-align:center;">${emptyText}</td></tr>`;
        } else {
            items.forEach(([k, v]) => {
                const attrK = k.replace(/"/g, '&quot;').replace(/'/g, '&#39;');
                html += `<tr>
                    <td>${escapeHtml(k)}</td>
                    <td><input type="text" value="${escapeHtml(v)}" data-key="${attrK}" class="dict-input" style="width:100%; box-sizing:border-box; background:transparent; color:inherit; border:1px solid #555; padding:4px;" autocomplete="off"></td>
                    <td>
                        ${dictType === 'user' ? `<button class="small-btn delete-item" data-key="${attrK}" style="background-color:#aa1111; color:#fff; padding:4px 8px;">🗑</button>` : ''}
                    </td>
                </tr>`;
            });
        }
        html += '</tbody></table>';
        if (dictTableContainer) dictTableContainer.innerHTML = html;

        document.querySelectorAll('.dict-input').forEach((dictInputEl) => {
            dictInputEl.addEventListener('change', async () => {
                const dictKey = dictInputEl.getAttribute('data-key');
                const dictValue = dictInputEl.value;
                await invoke('edit_dictionary_item', { key: dictKey, value: dictValue, delete: false });
                const mask = state.currentLabels.status_dict_item_updated || '已更新 {}';
                appendLog(mask.replace('{}', dictKey));
                loadDictionary();
            });
        });

        document.querySelectorAll('.delete-item').forEach((b) =>
            b.addEventListener('click', async (e) => {
                const key = e.currentTarget.getAttribute('data-key');
                const confirmMask = state.currentLabels.status_dict_item_delete_confirm;
                if (confirm(confirmMask.replace('{}', key))) {
                    await invoke('edit_dictionary_item', { key: key, value: '', delete: true });
                    loadDictionary();
                }
            })
        );
    } catch (e) {
        const mask = state.currentLabels.status_dict_load_failed || '讀取字典失敗 {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

export function initDictionary() {
    const btnNavDict = document.getElementById('btn-nav-dict');
    const tabUser = document.getElementById('tab-user');
    const tabOfficial = document.getElementById('tab-official');
    const dictSearch = document.getElementById('dict-search');
    const pagePrev = document.getElementById('page-prev');
    const pageNext = document.getElementById('page-next');
    const dictUserControls = document.getElementById('dict-user-controls');

    // Actions
    const btnDictAdd = document.getElementById('btn-dict-add');
    const btnDictReplace = document.getElementById('btn-dict-replace');
    const dictInputKey = document.getElementById('dict-input-key');
    const dictInputValue = document.getElementById('dict-input-value');
    const btnDictClear = document.getElementById('btn-dict-clear');
    const btnDictImport = document.getElementById('btn-dict-import');
    const btnDictExport = document.getElementById('btn-dict-export');
    const btnDictOpenJson = document.getElementById('btn-dict-open-json');

    if (btnNavDict) {
        btnNavDict.addEventListener('click', async () => {
            try {
                await invoke('open_dict_window');
            } catch (e) {
                console.error('開啟字典視窗失敗:', e);
            }
        });
    }

    if (window.__TAURI__ && window.__TAURI__.event) {
        // 📢 監聽字典變動事件進行多視窗同步
        window.__TAURI__.event.listen('dictionary-changed', () => {
            loadDictionary();
        });
    }
    if (tabUser)
        tabUser.addEventListener('click', () => {
            dictType = 'user';
            tabUser.classList.add('active');
            if (tabOfficial) tabOfficial.classList.remove('active');
            dictPage = 0;
            loadDictionary();

            // 顯示使用者編輯元件
            if (dictUserControls) dictUserControls.classList.toggle('hidden', false);
        });
    if (tabOfficial)
        tabOfficial.addEventListener('click', () => {
            dictType = 'official';
            tabOfficial.classList.add('active');
            if (tabUser) tabUser.classList.remove('active');
            dictPage = 0;
            loadDictionary();

            // 隱藏使用者編輯元件 (官方不可直接編輯)
            if (dictUserControls) dictUserControls.classList.toggle('hidden', true);
        });
    if (dictSearch)
        dictSearch.addEventListener('input', () => {
            dictPage = 0;
            loadDictionary();
        });

    const chkPriority = document.getElementById('chk-glossary-priority');
    if (chkPriority) {
        chkPriority.addEventListener('change', () => {
            dictPage = 0; // 重置頁碼
            if (typeof updateToggleStateLabel === 'function') {
                updateToggleStateLabel('chk-glossary-priority', chkPriority.checked);
            }
            loadDictionary();
        });
    }
    if (pagePrev)
        pagePrev.addEventListener('click', () => {
            if (dictPage > 0) {
                dictPage--;
                loadDictionary();
            }
        });
    if (pageNext)
        pageNext.addEventListener('click', () => {
            dictPage++;
            loadDictionary();
        });

    if (btnDictAdd && dictInputKey && dictInputValue) {
        btnDictAdd.addEventListener('click', async () => {
            const dictKey = dictInputKey.value.trim();
            const dictValue = dictInputValue.value.trim();
            if (!dictKey) return alert(state.currentLabels.status_dict_key_empty);
            try {
                await invoke('edit_dictionary_item', { key: dictKey, value: dictValue, delete: false });
                appendLog(state.currentLabels.status_dict_add_success.replace('{}', dictKey).replace('{}', dictValue));
                dictInputKey.value = '';
                dictInputValue.value = '';
                loadDictionary();
            } catch (e) {
                appendLog(state.currentLabels.status_dict_add_failed.replace('{}', state.currentLabels[e] || e));
            }
        });
    }

    if (btnDictReplace && dictInputKey && dictInputValue) {
        btnDictReplace.addEventListener('click', async () => {
            const oldV = dictInputKey.value.trim();
            const newV = dictInputValue.value.trim();
            if (!oldV || !newV) return alert(state.currentLabels.status_dict_replace_empty);
            if (confirm(state.currentLabels.status_dict_replace_confirm.replace('{}', oldV).replace('{}', newV))) {
                try {
                    await invoke('edit_dictionary_item', { key: oldV, value: newV, delete: false });
                    appendLog(state.currentLabels.status_dict_replace_sent.replace('{}', oldV).replace('{}', newV));
                    dictInputKey.value = '';
                    dictInputValue.value = '';
                    loadDictionary();
                } catch (e) {
                    appendLog(
                        state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e)
                    );
                }
            }
        });
    }

    if (btnDictClear) {
        btnDictClear.addEventListener('click', async () => {
            if (dictType !== 'user') return;
            if (confirm(state.currentLabels.glossary_clear_title)) {
                try {
                    await invoke('clear_user_dictionary');
                    appendLog(state.currentLabels.status_dict_clear_success);
                    loadDictionary();
                } catch (e) {
                    appendLog(
                        state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e)
                    );
                }
            }
        });
    }

    if (btnDictImport) {
        btnDictImport.addEventListener('click', async () => {
            if (dictType !== 'user') return;
            try {
                const path = await invoke('open_path_dialog', { diagType: 'file' });
                if (path) {
                    await invoke('import_user_dictionary', { filePath: path });
                    appendLog(state.currentLabels.status_dict_import_success);
                    loadDictionary();
                }
            } catch (e) {
                appendLog(state.currentLabels.status_dict_add_failed.replace('{}', state.currentLabels[e] || e));
            }
        });
    }

    if (btnDictExport) {
        btnDictExport.addEventListener('click', async () => {
            try {
                const path = await invoke('open_path_dialog', { diagType: 'save_file' });
                if (path) {
                    const exportPath = path.endsWith('.json') ? path : path + '.json';
                    await invoke('export_user_dictionary', { filePath: exportPath });
                    appendLog(state.currentLabels.status_dict_export_success.replace('{}', exportPath));
                }
            } catch (e) {
                appendLog(state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e));
            }
        });
    }

    if (btnDictOpenJson) {
        btnDictOpenJson.addEventListener('click', async () => {
            try {
                // dictType 在模組頂部有宣告
                await invoke('open_dictionary_location', { dictType: dictType });
            } catch (e) {
                appendLog(state.currentLabels.status_open_path_failed.replace('{}', e));
            }
        });
    }
}

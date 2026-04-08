// frontend/modules/dictionary.js
import { state } from './state.js';
import { appendLog, escapeHtml } from './utils.js';
import { updateToggleStateLabel } from './i18n.js';
import { dom } from './dom.js';

// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);

let dictPage = 0;
let dictPageSize = 10;
let dictType = 'user';

export async function loadDictionary() {
    try {
        const [items, totalPages] = await invoke('query_dictionary', {
            dictType: dictType,
            page: dictPage,
            pageSize: dictPageSize,
            searchKey: dom.dictSearch ? dom.dictSearch.value.trim() : '',
        });

        if (dom.pageInfo) {
            const mask = state.currentLabels.label_page_info || '第 {} / {} 頁';
            const parts = mask.split('{}');
            if (parts.length >= 3) {
                dom.pageInfo.textContent = `${parts[0]}${dictPage + 1}${parts[1]}${totalPages || 1}${parts[2]}`;
            } else {
                dom.pageInfo.textContent = mask.replace('{}', dictPage + 1).replace('{}', totalPages || 1);
            }
        }
        if (dom.pagePrev) dom.pagePrev.disabled = dictPage === 0;
        if (dom.pageNext) dom.pageNext.disabled = totalPages === 0 || dictPage + 1 >= totalPages;

        const colKey = (state.currentLabels.glossary_key || '原文').replace(':', '');
        const colVal = (state.currentLabels.glossary_value || '翻譯').replace(':', '');
        const emptyText = state.currentLabels.glossary_empty || '無資料';

        let html = `<table class="dict-table"><thead><tr><th>${colKey}</th><th>${colVal}</th></tr></thead><tbody>`;
        if (!items || items.length === 0) {
            html += `<tr><td colspan="2" style="text-align:center;">${emptyText}</td></tr>`;
        } else {
            items.forEach(([k, v]) => {
                const attrK = k.replace(/"/g, '&quot;').replace(/'/g, '&#39;');
                html += `<tr>
                    <td>${escapeHtml(k)}</td>
                    <td><input type="text" value="${escapeHtml(v)}" data-key="${attrK}" class="dict-input" style="width:100%; box-sizing:border-box; background:transparent; color:inherit; border:1px solid #555; padding:4px;" autocomplete="off"></td>
                </tr>`;
            });
        }
        html += '</tbody></table>';
        if (dom.dictTableContainer) dom.dictTableContainer.innerHTML = html;

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
    } catch (e) {
        const mask = state.currentLabels.status_dict_load_failed || '讀取字典失敗 {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e));
    }
}

export function initDictionary() {
    if (dom.btnNavDict) {
        dom.btnNavDict.addEventListener('click', async () => {
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
    if (dom.tabUser)
        dom.tabUser.addEventListener('click', () => {
            dictType = 'user';
            dom.tabUser.classList.add('active');
            if (dom.tabOfficial) dom.tabOfficial.classList.remove('active');
            dictPage = 0;
            loadDictionary();

            // 顯示使用者編輯元件
            if (dom.dictUserControls) dom.dictUserControls.classList.toggle('hidden', false);
        });
    if (dom.tabOfficial)
        dom.tabOfficial.addEventListener('click', () => {
            dictType = 'official';
            dom.tabOfficial.classList.add('active');
            if (dom.tabUser) dom.tabUser.classList.remove('active');
            dictPage = 0;
            loadDictionary();

            // 隱藏使用者編輯元件 (官方不可直接編輯)
            if (dom.dictUserControls) dom.dictUserControls.classList.toggle('hidden', true);
        });
    if (dom.dictSearch)
        dom.dictSearch.addEventListener('input', () => {
            dictPage = 0;
            loadDictionary();
        });

    if (dom.chkGlossaryPriority) {
        dom.chkGlossaryPriority.addEventListener('change', () => {
            dictPage = 0; // 重置頁碼
            if (typeof updateToggleStateLabel === 'function') {
                updateToggleStateLabel('chk-glossary-priority', dom.chkGlossaryPriority.checked);
            }
            loadDictionary();
        });
    }
    if (dom.pagePrev)
        dom.pagePrev.addEventListener('click', () => {
            if (dictPage > 0) {
                dictPage--;
                loadDictionary();
            }
        });
    if (dom.pageNext)
        dom.pageNext.addEventListener('click', () => {
            dictPage++;
            loadDictionary();
        });

    if (dom.btnDictAdd && dom.dictInputKey && dom.dictInputValue) {
        dom.btnDictAdd.addEventListener('click', async () => {
            const dictKey = dom.dictInputKey.value.trim();
            const dictValue = dom.dictInputValue.value.trim();
            if (!dictKey) return alert(state.currentLabels.status_dict_key_empty);
            try {
                await invoke('edit_dictionary_item', { key: dictKey, value: dictValue, delete: false });
                appendLog(state.currentLabels.status_dict_add_success.replace('{}', dictKey).replace('{}', dictValue));
                dom.dictInputKey.value = '';
                dom.dictInputValue.value = '';
                loadDictionary();
            } catch (e) {
                appendLog(state.currentLabels.status_dict_add_failed.replace('{}', state.currentLabels[e] || e));
            }
        });
    }

    if (dom.btnDictReplace && dom.dictInputKey && dom.dictInputValue) {
        dom.btnDictReplace.addEventListener('click', async () => {
            const oldV = dom.dictInputKey.value.trim();
            const newV = dom.dictInputValue.value.trim();
            if (!oldV || !newV) return alert(state.currentLabels.status_dict_replace_empty);
            if (confirm(state.currentLabels.status_dict_replace_confirm.replace('{}', oldV).replace('{}', newV))) {
                try {
                    await invoke('edit_dictionary_item', { key: oldV, value: newV, delete: false });
                    appendLog(state.currentLabels.status_dict_replace_sent.replace('{}', oldV).replace('{}', newV));
                    dom.dictInputKey.value = '';
                    dom.dictInputValue.value = '';
                    loadDictionary();
                } catch (e) {
                    appendLog(
                        state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e)
                    );
                }
            }
        });
    }

    if (dom.btnDictClear) {
        dom.btnDictClear.addEventListener('click', async () => {
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

    if (dom.btnDictImport) {
        dom.btnDictImport.addEventListener('click', async () => {
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

    if (dom.btnDictExport) {
        dom.btnDictExport.addEventListener('click', async () => {
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

    if (dom.btnDictOpenJson) {
        dom.btnDictOpenJson.addEventListener('click', async () => {
            try {
                await invoke('open_dictionary_location', { dictType: dictType });
            } catch (e) {
                appendLog(state.currentLabels.status_open_path_failed.replace('{}', e));
            }
        });
    }
}

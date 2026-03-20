// frontend/modules/dictionary.js
import { state } from './state.js';
import { appendLog } from './utils.js';

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
            dictType: dictType, page: dictPage, pageSize: dictPageSize, searchKey: dictSearch ? dictSearch.value.trim() : ''
        });

        if (pageInfo) {
            const mask = state.currentLabels.label_page_info || '第 {} / {} 頁';
            pageInfo.textContent = mask.replace('{}', dictPage + 1).replace('{}', totalPages || 1);
        }
        if (pagePrev) pagePrev.disabled = dictPage === 0;
        if (pageNext) pageNext.disabled = totalPages === 0 || dictPage + 1 >= totalPages;

        const colKey = state.currentLabels.glossary_key ? state.currentLabels.glossary_key.replace(':', '') : '原文 (Key)';
        const colVal = state.currentLabels.glossary_value ? state.currentLabels.glossary_value.replace(':', '') : '翻譯 (Value)';
        const colAct = state.currentLabels.glossary_col_actions || '操作';
        const emptyText = state.currentLabels.glossary_empty || '無結果';

        let html = `<table class="dict-table"><thead><tr><th>${colKey}</th><th>${colVal}</th><th>${colAct}</th></tr></thead><tbody>`;
        if (!items || items.length === 0) { html += `<tr><td colspan="3" style="text-align:center;">${emptyText}</td></tr>`; }
        else {
            items.forEach(([k, v]) => {
                const safeK = k.replace(/'/g, "&apos;").replace(/"/g, "&quot;");
                html += `<tr>
                    <td>${k}</td>
                    <td><input type="text" value="${v}" id="dict-val-${safeK}" class="dict-input" style="width:100%; box-sizing:border-box; background:transparent; color:inherit; border:1px solid #555; padding:4px;"></td>
                    <td>
                        <button class="small-btn save-item" data-key="${safeK}" style="padding:4px 8px;">💾</button>
                        ${dictType === 'user' ? `<button class="small-btn delete-item" data-key="${safeK}" style="background-color:#aa1111; color:#fff; padding:4px 8px;">🗑</button>` : ''}
                    </td>
                </tr>`;
            });
        }
        html += '</tbody></table>';
        if (dictTableContainer) dictTableContainer.innerHTML = html;

        document.querySelectorAll('.save-item').forEach(b => b.addEventListener('click', async (e) => {
            const key = e.currentTarget.getAttribute('data-key');
            const val = document.getElementById(`dict-val-${key}`).value;
            await invoke('edit_dictionary_item', { key: key, value: val, delete: false });
            const mask = state.currentLabels.status_dict_item_updated || '📖 字典更新：{}';
            appendLog(mask.replace('{}', key)); 
            loadDictionary();
        }));

        document.querySelectorAll('.delete-item').forEach(b => b.addEventListener('click', async (e) => {
            const key = e.currentTarget.getAttribute('data-key');
            const confirmMask = state.currentLabels.status_dict_item_delete_confirm || '確定刪除條目 {} 嗎？';
            if (confirm(confirmMask.replace('{}', key))) {
                await invoke('edit_dictionary_item', { key: key, value: '', delete: true });
                loadDictionary();
            }
        }));
    } catch (e) { 
        const mask = state.currentLabels.status_dict_load_failed || '❌ 載入字典失敗: {}';
        appendLog(mask.replace('{}', state.currentLabels[e] || e)); 
    }
}

export function initDictionary() {
    const btnNavDict = document.getElementById('btn-nav-dict');
    const dictDialog = document.getElementById('dict-dialog');
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

    if (btnNavDict && dictDialog) {
        btnNavDict.addEventListener('click', () => { dictPage = 0; dictDialog.showModal(); loadDictionary(); if (dictUserControls) dictUserControls.style.display = 'flex'; });
    }
    if (tabUser) tabUser.addEventListener('click', () => { dictType = 'user'; tabUser.classList.add('active'); if(tabOfficial) tabOfficial.classList.remove('active'); dictPage = 0; loadDictionary(); if (dictUserControls) dictUserControls.style.display = 'flex'; });
    if (tabOfficial) tabOfficial.addEventListener('click', () => { dictType = 'official'; tabOfficial.classList.add('active'); if(tabUser) tabUser.classList.remove('active'); dictPage = 0; loadDictionary(); if (dictUserControls) dictUserControls.style.display = 'none'; });
    if (dictSearch) dictSearch.addEventListener('input', () => { dictPage = 0; loadDictionary(); });
    if (pagePrev) pagePrev.addEventListener('click', () => { if (dictPage > 0) { dictPage--; loadDictionary(); } });
    if (pageNext) pageNext.addEventListener('click', () => { dictPage++; loadDictionary(); });

    if (btnDictAdd && dictInputKey && dictInputValue) {
        btnDictAdd.addEventListener('click', async () => {
            const k = dictInputKey.value.trim(); const v = dictInputValue.value.trim();
            if (!k) return alert(state.currentLabels.status_dict_key_empty);
            try {
                await invoke('edit_dictionary_item', { key: k, value: v, delete: false });
                appendLog(state.currentLabels.status_dict_add_success.replace('{}', k).replace('{}', v));
                dictInputKey.value = ''; dictInputValue.value = ''; loadDictionary();
            } catch (e) { appendLog(state.currentLabels.status_dict_add_failed.replace('{}', state.currentLabels[e] || e)); }
        });
    }

    if (btnDictReplace && dictInputKey && dictInputValue) {
        btnDictReplace.addEventListener('click', async () => {
            const oldV = dictInputKey.value.trim(); const newV = dictInputValue.value.trim();
            if (!oldV || !newV) return alert(state.currentLabels.status_dict_replace_empty);
            if (confirm(state.currentLabels.status_dict_replace_confirm.replace('{}', oldV).replace('{}', newV))) {
                try {
                    await invoke('edit_dictionary_item', { key: oldV, value: newV, delete: false });
                    appendLog(state.currentLabels.status_dict_replace_sent.replace('{}', oldV).replace('{}', newV));
                    dictInputKey.value = ''; dictInputValue.value = ''; loadDictionary();
                } catch (e) { appendLog(state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e)); }
            }
        });
    }

    if (btnDictClear) {
        btnDictClear.addEventListener('click', async () => {
            if (dictType !== 'user') return;
            if (confirm(state.currentLabels.glossary_clear_title || '確定清空全部？')) {
                try { await invoke('clear_user_dictionary'); appendLog(state.currentLabels.status_dict_clear_success); loadDictionary(); }
                catch (e) { appendLog(state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e)); }
            }
        });
    }

    if (btnDictImport) {
        btnDictImport.addEventListener('click', async () => {
            if (dictType !== 'user') return;
            try {
                const path = await invoke('open_path_dialog', { diagType: 'file' });
                if (path) { await invoke('import_user_dictionary', { filePath: path }); appendLog(state.currentLabels.status_dict_import_success); loadDictionary(); }
            } catch (e) { appendLog(state.currentLabels.status_dict_add_failed.replace('{}', state.currentLabels[e] || e)); }
        });
    }

    if (btnDictExport) {
        btnDictExport.addEventListener('click', async () => {
            try {
                const path = await invoke('open_path_dialog', { diagType: 'save_file' });
                if (path) { const p = path.endsWith('.json') ? path : path + '.json'; await invoke('export_user_dictionary', { filePath: p }); appendLog(state.currentLabels.status_dict_export_success.replace('{}', p)); }
            } catch (e) { appendLog(state.currentLabels.status_dict_replace_failed.replace('{}', state.currentLabels[e] || e)); }
        });
    }
}

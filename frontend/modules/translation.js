// frontend/modules/translation.js
import { state } from './state.js';
import { appendLog } from './utils.js';
import { dom } from './dom.js';

// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);
const { listen } = window.__TAURI__ ? window.__TAURI__.event : { listen: () => {} };

// 狀態常量 (與後端 SCREAMING_SNAKE_CASE 映射)
const UI_STATUS = {
    IDLE: 'IDLE',
    RUNNING: 'RUNNING',
    PAUSED: 'PAUSED',
};

async function getFormConfig() {
    return invoke('build_form_config_cmd', {
        base: state.currentConfig,
        input: {
            api_provider: dom.apiProvider?.value || '無',
            api_base_url: dom.apiBaseUrl?.value || '',
            ollama_url: dom.ollamaUrl?.value || 'http://localhost:11434',
            model: dom.selectedModel?.value || '',
            source_lang: dom.sourceLang?.value || 'en_us',
            target_lang: dom.targetLang?.value || 'zh_tw',
            batch_size: dom.batchSize?.value || '',
            batch_max_chars: dom.batchMaxChars?.value || '',
            timeout: dom.timeoutSec?.value || '',
            output_dir: dom.outputDir?.value || '',
            pack_format: dom.packFormat?.value || '',
            user_prompt: dom.userPrompt?.value || '',
            system_prompt: dom.systemPrompt?.value || '',
            glossary_priority: dom.chkGlossaryPriority?.checked ? 'user' : 'official',
            skip_json: dom.chkSkipJson?.checked || false,
            skip_js: dom.chkSkipJs?.checked || false,
            skip_jar: dom.chkSkipJar?.checked || false,
            skip_book: dom.chkSkipBook?.checked || false,
            enable_llm_log: dom.chkLlmLog?.checked || false,
            enable_debug_log: dom.chkDebugLog?.checked || false,
            show_debug_tools: dom.chkDebugTools?.checked || false,
            ui_lang: dom.uiLang?.value || 'zh_tw',
            path: dom.inputPath?.value || '',
            fast_convert: dom.chkFastConvert?.checked || false,
            excluded_paths_text: dom.excludedPaths?.value || '',
        },
    });
}

/**
 * 核心：根據狀態更新 UI 元件鎖定與顯示
 */
function applyUiPatch(patch) {
    const lockControls = !!patch.lock_controls;
    const showTranslate = !!patch.show_translate;
    const showPause = !!patch.show_pause;
    const showResume = !!patch.show_resume;
    const showStop = !!patch.show_stop;

    // 1. 元件鎖定
    const lockedSelectors = [
        '#input-path',
        '#btn-browse-file',
        '#btn-browse-dir',
        '#btn-browse-output',
        '.input-row .clear-btn',
        '#output-dir',
        '.api-settings input',
        '.api-settings select',
        '.trans-params input',
        '.trans-params select',
        '.trans-params textarea',
        '#developer-settings input',
        '#developer-settings select',
        '#developer-settings textarea',
        '#user-prompt',
        '#system-prompt',
    ].join(',');

    const elementsToLock = document.querySelectorAll(lockedSelectors);
    elementsToLock.forEach((el) => {
        el.disabled = lockControls;
    });

    if (dom.btnTranslate) dom.btnTranslate.style.display = showTranslate ? 'inline-block' : 'none';
    if (dom.btnPause) dom.btnPause.style.display = showPause ? 'inline-block' : 'none';
    if (dom.btnResume) dom.btnResume.style.display = showResume ? 'inline-block' : 'none';
    if (dom.btnStop) dom.btnStop.style.display = showStop ? 'inline-block' : 'none';

    let notice = document.getElementById('pause-notice');
    if (!notice && (showPause || showResume || showStop)) {
        notice = document.createElement('div');
        notice.id = 'pause-notice';
        notice.style.fontSize = '12px';
        notice.style.color = 'var(--accent-color, #ffaa00)';
        notice.style.marginTop = '5px';
        notice.style.transition = 'opacity 0.3s';
        const ctrlPanel = document.querySelector('.control-panel .actions');
        if (ctrlPanel) ctrlPanel.appendChild(notice);
    }
    if (notice) {
        notice.textContent = patch.pause_notice || '';
        notice.style.opacity = patch.pause_notice ? '1' : '0';
    }

    if (patch.status !== UI_STATUS.RUNNING) {
        const pulses = document.querySelectorAll('.pulse-glow, [style*="pulse"]');
        pulses.forEach((el) => (el.style.animation = 'none'));
    }

    if (patch.clear_batch_status) {
        if (dom.batchStatusText) dom.batchStatusText.textContent = '';
    }
    if (patch.clear_current_status) {
        if (dom.currentStatusLabel) dom.currentStatusLabel.textContent = '';
    }
}

export async function updateUiState(status) {
    const patch = await invoke('derive_ui_state', { status, lang: dom.uiLang?.value });
    applyUiPatch(patch || { status });
}

export function initTranslation() {
    if (dom.btnTranslate) {
        dom.btnTranslate.addEventListener('click', async () => {
            if (dom.inputPath && dom.inputPath.value.trim() === '') {
                return alert(state.currentLabels.status_input_path_empty || '請先選擇輸入路徑');
            }
            try {
                // 更新當前 Config Snapshot
                state.currentConfig = await getFormConfig();
                const inputPath = dom.inputPath.value;

                updateUiState(UI_STATUS.RUNNING);

                if (dom.progressBar) dom.progressBar.style.width = '0%';
                if (dom.batchProgressBar) dom.batchProgressBar.style.width = '0%';
                if (dom.statusText) dom.statusText.textContent = state.currentLabels.status_trans_starting;

                await invoke('start_translation', {
                    config: state.currentConfig,
                    inputPaths: [inputPath],
                });
            } catch (e) {
                appendLog({
                    level: 'Error',
                    message: (state.currentLabels.status_trans_failed_mask || '翻譯失敗: {}').replace('{}', e),
                    timestamp: Date.now(),
                });
                updateUiState(UI_STATUS.IDLE);
            }
        });
    }

    if (dom.btnPause) {
        dom.btnPause.addEventListener('click', async () => {
            try {
                await invoke('pause_translation');
                if (dom.statusText) dom.statusText.textContent = state.currentLabels.status_trans_paused;
            } catch (e) {
                console.error('Pause failed:', e);
            }
        });
    }

    if (dom.btnResume) {
        dom.btnResume.addEventListener('click', async () => {
            try {
                // 1. 先同步 UI 修改至後端
                const latestConfig = await getFormConfig();
                await invoke('update_active_job_config', { config: latestConfig });

                // 2. 執行恢復
                await invoke('resume_translation');
                if (dom.statusText) dom.statusText.textContent = state.currentLabels.status_trans_resumed;
            } catch (e) {
                console.error('Resume failed:', e);
            }
        });
    }

    if (dom.btnStop) {
        dom.btnStop.addEventListener('click', async () => {
            const confirmed = window.confirm(state.currentLabels.text_confirm_stop || '確定要停止翻譯嗎？');
            if (confirmed) {
                try {
                    await invoke('stop_translation');
                    if (dom.statusText) dom.statusText.textContent = state.currentLabels.status_trans_stopping;
                } catch (e) {
                    console.error('Stop failed:', e);
                }
            }
        });
    }

    // --- 實例化 Listeners ---
    if (window.__TAURI__) {
        // 監聽後端狀態同步 (核心)
        listen('job-state-changed', (event) => {
            console.log('Job state changed:', event.payload);
            updateUiState(event.payload);
        });

        listen('translation-progress', (event) => {
            const data = event.payload;
            if (dom.progressBar && data.total > 0) {
                const pct = (data.current / data.total) * 100;
                dom.progressBar.style.width = `${pct}%`;
            }
            if (dom.currentStatusLabel && data.msg) {
                dom.currentStatusLabel.textContent = data.msg;
            }
            if (dom.statusText && data.total > 0) {
                const pct = (data.current / data.total) * 100;
                let progressText = `${Math.round(pct)}%`;
                if (state.currentLabels.status_progress_detailed_mask) {
                    progressText = state.currentLabels.status_progress_detailed_mask
                        .replace('{}', data.current)
                        .replace('{}', data.total)
                        .replace('{}', `${Math.round(pct)}%`);
                }
                dom.statusText.textContent = progressText;
            }
        });

        listen('translation-finished', (event) => {
            const data = event.payload;
            // updateUiState 會由 job-state-changed: IDLE 觸發，此處僅做掃尾
            if (data.success) {
                if (dom.progressBar) dom.progressBar.style.width = '100%';
                if (dom.batchProgressBar) dom.batchProgressBar.style.width = '100%';
            }

            if (dom.statusText)
                dom.statusText.textContent = data.success
                    ? state.currentLabels.status_finished
                    : state.currentLabels.status_failed_or_cancelled;

            appendLog({
                level: data.success ? 'Success' : 'Error',
                message: data.msg,
                timestamp: Date.now(),
            });
        });

        listen('translation-batch-update', (event) => {
            const data = event.payload;
            if (dom.batchProgressBar && data.total_batches > 0) {
                const pct = (data.batch_index / data.total_batches) * 100;
                dom.batchProgressBar.style.width = `${pct}%`;
                if (dom.batchProgressBar.nextElementSibling) {
                    dom.batchProgressBar.nextElementSibling.style.animation = 'pulse 1.5s infinite';
                }
            }
            if (dom.batchStatusText) {
                const mask = state.currentLabels.status_batch_mask || '批次 {}/{}';
                dom.batchStatusText.textContent = mask
                    .replace('{}', data.batch_index)
                    .replace('{}', data.total_batches);
            }
        });

        listen('translation-log', (event) => {
            appendLog(event.payload);
        });
    }
}

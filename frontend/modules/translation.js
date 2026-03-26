// frontend/modules/translation.js
import { state } from './state.js';
import { appendLog } from './utils.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };
const { listen } = window.__TAURI__ ? window.__TAURI__.event : { listen: () => {} };

// 狀態常量 (與後端 SCREAMING_SNAKE_CASE 映射)
const UI_STATUS = {
    IDLE: 'IDLE',
    RUNNING: 'RUNNING',
    PAUSED: 'PAUSED'
};

/**
 * 從 DOM 獲取當前所有的表單配置值
 */
function getFormConfig() {
    return {
        api_provider: document.getElementById('api-provider')?.value || '無',
        api_base_url: document.getElementById('api-base-url')?.value || '',
        ollama_url: document.getElementById('ollama-url')?.value || 'http://localhost:11434',
        model: document.getElementById('selected-model')?.value || '',
        source_lang: document.getElementById('source-lang')?.value || 'en_us',
        target_lang: document.getElementById('target-lang')?.value || 'zh_tw',
        batch_size: parseInt(document.getElementById('batch-size')?.value || '10'),
        batch_max_chars: parseInt(document.getElementById('batch-max-chars')?.value || '1000'),
        timeout: parseInt(document.getElementById('timeout-sec')?.value || '30'),
        output_dir: document.getElementById('output-dir')?.value || '',
        pack_format: parseInt(document.getElementById('pack-format')?.value || '15'),
        user_prompt: document.getElementById('user-prompt')?.value || '',
        system_prompt: document.getElementById('system-prompt')?.value || '',
        glossary_priority: document.getElementById('chk-glossary-priority')?.checked ? 'user' : 'official',
        skip_json: document.getElementById('chk-skip-json')?.checked || false,
        skip_js: document.getElementById('chk-skip-js')?.checked || false,
        skip_jar: document.getElementById('chk-skip-jar')?.checked || false,
        skip_book: document.getElementById('chk-skip-book')?.checked || false,
        enable_llm_log: document.getElementById('chk-llm-log')?.checked || false,
        enable_debug_log: document.getElementById('chk-debug-log')?.checked || false,
        ui_lang: document.getElementById('ui-lang')?.value || 'zh_tw'
    };
}

/**
 * 核心：根據狀態更新 UI 元件鎖定與顯示
 */
export function updateUiState(status) {
    const isRunning = status === UI_STATUS.RUNNING;
    const isPaused = status === UI_STATUS.PAUSED;
    const isIdle = status === UI_STATUS.IDLE;

    // 1. 元件鎖定 (嚴格對齊對照表)
    // 鎖定範圍：API 設定、參數、開發者選項、路徑輸入框
    const lockedSelectors = [
        '#input-path',
        '.action-btn',
        '#output-dir',
        '.api-settings input',
        '.api-settings select',
        '.trans-params input',
        '.trans-params select',
        '.trans-params textarea',
        '.dev-settings input',
        '.dev-settings textarea',
        '#user-prompt',
        '#system-prompt'
    ].join(',');

    const elementsToLock = document.querySelectorAll(lockedSelectors);
    elementsToLock.forEach(el => {
        // 唯有在 RUNNING 時鎖定，IDLE 與 PAUSED 皆開放
        el.disabled = isRunning;
    });

    // 2. 按鈕顯隱控制
    const btnTranslate = document.getElementById('btn-translate');
    const btnPause = document.getElementById('btn-pause');
    const btnResume = document.getElementById('btn-resume');
    const btnStop = document.getElementById('btn-stop');

    if (btnTranslate) btnTranslate.style.display = isIdle ? 'inline-block' : 'none';
    if (btnPause) btnPause.style.display = isRunning ? 'inline-block' : 'none';
    if (btnResume) btnResume.style.display = isPaused ? 'inline-block' : 'none';
    if (btnStop) btnStop.style.display = isPaused ? 'inline-block' : 'none';

    // 3. 暫停提示訊息
    let notice = document.getElementById('pause-notice');
    if (!notice && (isRunning || isPaused)) {
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
        notice.textContent = isPaused ? '* 修改設定將在恢復後的下一個批次生效' : '';
        notice.style.opacity = isPaused ? '1' : '0';
    }

    // 4. 動畫與狀態清理
    if (isIdle || isPaused) {
        const pulses = document.querySelectorAll('.pulse-glow, [style*="pulse"]');
        pulses.forEach(el => el.style.animation = 'none');
    }

    if (isIdle) {
        // 重置狀態標籤
        const batchText = document.getElementById('batch-status-text');
        if (batchText) batchText.textContent = '';
        const currentStatusLabel = document.getElementById('current-status-label');
        if (currentStatusLabel) currentStatusLabel.textContent = '';
    }
}

// 舊函式相容性包裝
export function setRunningState(isRunning) {
    updateUiState(isRunning ? UI_STATUS.RUNNING : UI_STATUS.IDLE);
}

export function initTranslation() {
    const btnTranslate = document.getElementById('btn-translate');
    const btnPause = document.getElementById('btn-pause');
    const btnResume = document.getElementById('btn-resume');
    const btnStop = document.getElementById('btn-stop');
    const progressBar = document.getElementById('progress-bar');
    const statusText = document.getElementById('status-text');

    if (btnTranslate) {
        btnTranslate.addEventListener('click', async () => {
            const inputPath = document.getElementById('input-path');
            if (inputPath && inputPath.value.trim() === '') {
                return alert(state.currentLabels.status_input_path_empty || '請先選擇輸入路徑');
            }
            try {
                // 更新當前 Config Snapshot
                state.currentConfig = getFormConfig();
                state.currentConfig.path = inputPath.value;

                updateUiState(UI_STATUS.RUNNING);

                if (progressBar) progressBar.style.width = '0%';
                const batchProgress = document.getElementById('batch-progress-bar');
                if (batchProgress) batchProgress.style.width = '0%';
                if (statusText) statusText.textContent = state.currentLabels.status_trans_starting;

                await invoke('start_translation', {
                    config: state.currentConfig,
                    inputPaths: [state.currentConfig.path]
                });
            } catch (e) {
                appendLog({
                    level: 'Error',
                    message: (state.currentLabels.status_trans_failed_mask || '翻譯失敗: {}').replace('{}', e),
                    timestamp: Date.now()
                });
                updateUiState(UI_STATUS.IDLE);
            }
        });
    }

    if (btnPause) {
        btnPause.addEventListener('click', async () => {
            try {
                await invoke('pause_translation');
                if (statusText) statusText.textContent = state.currentLabels.status_trans_paused;
            } catch (e) {
                console.error("Pause failed:", e);
            }
        });
    }

    if (btnResume) {
        btnResume.addEventListener('click', async () => {
            try {
                // 1. 先同步 UI 修改至後端
                const latestConfig = getFormConfig();
                await invoke('update_active_job_config', { config: latestConfig });

                // 2. 執行恢復
                await invoke('resume_translation');
                if (statusText) statusText.textContent = state.currentLabels.status_trans_resumed;
            } catch (e) {
                console.error("Resume failed:", e);
            }
        });
    }

    if (btnStop) {
        btnStop.addEventListener('click', async () => {
            // 彈出確認對話框
            const confirmed = window.confirm(state.currentLabels.text_confirm_stop || '確定要停止翻譯嗎？');
            if (confirmed) {
                try {
                    await invoke('stop_translation');
                    if (statusText) statusText.textContent = state.currentLabels.status_trans_stopping;
                } catch (e) {
                    console.error("Stop failed:", e);
                }
            }
        });
    }

    // --- 實例化 Listeners ---
    if (window.__TAURI__) {
        // 監聽後端狀態同步 (核心)
        listen('job-state-changed', (event) => {
            console.log("Job state changed:", event.payload);
            updateUiState(event.payload);
        });

        listen('translation-progress', (event) => {
            const data = event.payload;
            if (progressBar && data.total > 0) {
                const pct = (data.current / data.total) * 100;
                progressBar.style.width = `${pct}%`;
            }
            const currentStatusLabel = document.getElementById('current-status-label');
            if (currentStatusLabel && data.msg) {
                currentStatusLabel.textContent = data.msg;
            }
            if (statusText && data.total > 0) {
                const pct = (data.current / data.total) * 100;
                let progressText = `${Math.round(pct)}%`;
                if (state.currentLabels.status_progress_detailed_mask) {
                    progressText = state.currentLabels.status_progress_detailed_mask
                        .replace('{}', data.current)
                        .replace('{}', data.total)
                        .replace('{}', `${Math.round(pct)}%`);
                }
                statusText.textContent = progressText;
            }
        });

        listen('translation-finished', (event) => {
            const data = event.payload;
            // updateUiState 會由 job-state-changed: IDLE 觸發，此處僅做掃尾
            if (data.success) {
                if (progressBar) progressBar.style.width = '100%';
                const batchProgress = document.getElementById('batch-progress-bar');
                if (batchProgress) batchProgress.style.width = '100%';
            }

            if (statusText)
                statusText.textContent = data.success
                    ? state.currentLabels.status_finished
                    : state.currentLabels.status_failed_or_cancelled;

            appendLog({
                level: data.success ? 'Success' : 'Error',
                message: data.msg,
                timestamp: Date.now()
            });
        });

        listen('translation-batch-update', (event) => {
            const data = event.payload;
            const batchProgress = document.getElementById('batch-progress-bar');
            const batchText = document.getElementById('batch-status-text');
            if (batchProgress && data.total_batches > 0) {
                const pct = (data.batch_index / data.total_batches) * 100;
                batchProgress.style.width = `${pct}%`;
                if (batchProgress.nextElementSibling) {
                    batchProgress.nextElementSibling.style.animation = 'pulse 1.5s infinite';
                }
            }
            if (batchText) {
                const mask = state.currentLabels.status_batch_mask || '批次 {}/{}';
                batchText.textContent = mask.replace('{}', data.batch_index).replace('{}', data.total_batches);
            }
        });

        listen('translation-log', (event) => {
            appendLog(event.payload);
        });
    }
}

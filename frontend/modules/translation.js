// frontend/modules/translation.js
import { state } from './state.js';
import { appendLog } from './utils.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };
const { listen } = window.__TAURI__ ? window.__TAURI__.event : { listen: () => {} };

export function setRunningState(isRunning) {
    const btnTranslate = document.getElementById('btn-translate');
    const btnPause = document.getElementById('btn-pause');
    const btnResume = document.getElementById('btn-resume');
    const btnStop = document.getElementById('btn-stop');

    const inputs = document.querySelectorAll(
        '.control-panel:not(.theme-settings) input:not(#input-path), .control-panel:not(.theme-settings) select, .control-panel:not(.theme-settings) textarea'
    );
    inputs.forEach((el) => (el.disabled = isRunning));

    if (btnTranslate && btnPause && btnStop && btnResume) {
        if (isRunning) {
            btnTranslate.style.display = 'none';
            btnPause.style.display = 'inline-block';
            btnStop.style.display = 'inline-block';
            btnPause.textContent = state.currentLabels.btn_pause;
        } else {
            btnTranslate.style.display = 'inline-block';
            btnPause.style.display = 'none';
            btnResume.style.display = 'none';
            btnStop.style.display = 'none';

            // 停止所有進度條動畫
            const pulses = document.querySelectorAll('.pulse-glow'); // 假設這是樣式類別
            pulses.forEach((el) => {
                el.style.animation = 'none';
            });
            // 同時檢查內連樣式
            const bars = [document.getElementById('progress-bar'), document.getElementById('batch-progress-bar')];
            bars.forEach(bar => {
                if (bar && bar.nextElementSibling) {
                    bar.nextElementSibling.style.animation = 'none';
                }
            });
        }
    }
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
            const outputDir = document.getElementById('output-dir');
            if (inputPath && inputPath.value.trim() === '') {
                return alert(state.currentLabels.status_input_path_empty);
            }
            // 移除強制選擇「輸出資料夾」的驗證
            try {
                // 確保從 DOM 取出最新狀態
                state.currentConfig.path = inputPath ? inputPath.value : '';

                let outDir = outputDir ? outputDir.value.trim() : '';
                state.currentConfig.output_dir = outDir;

                setRunningState(true);
                if (progressBar) {
                    progressBar.style.width = '0%';
                }
                const batchProgress = document.getElementById('batch-progress-bar');
                if (batchProgress) {
                    batchProgress.style.width = '0%';
                }
                if (statusText) statusText.textContent = state.currentLabels.status_trans_starting;

                await invoke('start_translation', {
                    config: state.currentConfig,
                    inputPaths: [state.currentConfig.path]
                });
            } catch (e) {
                appendLog({
                    level: 'Error',
                    message: state.currentLabels.status_trans_failed_mask.replace('{}', e),
                    timestamp: Date.now()
                });
                setRunningState(false);
            }
        });
    }

    if (btnPause) {
        btnPause.addEventListener('click', async () => {
            await invoke('pause_translation');
            if (btnPause && btnResume) {
                btnPause.style.display = 'none';
                btnResume.style.display = 'inline-block';
                btnResume.textContent = state.currentLabels.btn_resume;
            }
            if (statusText) statusText.textContent = state.currentLabels.status_trans_paused;
        });
    }

    if (btnResume) {
        btnResume.addEventListener('click', async () => {
            await invoke('resume_translation');
            if (btnPause && btnResume) {
                btnPause.style.display = 'inline-block';
                btnResume.style.display = 'none';
            }
            if (statusText) statusText.textContent = state.currentLabels.status_trans_resumed;
        });
    }

    if (btnStop) {
        btnStop.addEventListener('click', async () => {
            await invoke('stop_translation');
            if (statusText) statusText.textContent = state.currentLabels.status_trans_stopping;
        });
    }

    // --- 實例化 Listeners ---
    if (window.__TAURI__) {
        listen('translation-progress', (event) => {
            const data = event.payload; // { current: n, total: m, msg: "...", filename: "..." }
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
            // 狀態訊息僅顯示在狀態列，不寫入日誌區域
        });

        listen('translation-finished', (event) => {
            const data = event.payload; // { success: bool, msg: "..." }
            setRunningState(false);

            if (data.success) {
                if (progressBar) progressBar.style.width = '100%';
                const batchProgress = document.getElementById('batch-progress-bar');
                if (batchProgress) batchProgress.style.width = '100%';
            } else {
                if (progressBar) progressBar.style.width = '0%';
                const batchProgress = document.getElementById('batch-progress-bar');
                if (batchProgress) batchProgress.style.width = '0%';
            }

            // 結束時更新狀態
            if (statusText)
                statusText.textContent = data.success
                    ? state.currentLabels.status_finished
                    : state.currentLabels.status_failed_or_cancelled;

            // 清空批次狀態與當前處理路徑標籤 (避免殘留舊資訊)
            const batchText = document.getElementById('batch-status-text');
            if (batchText) batchText.textContent = '';
            const currentStatusLabel = document.getElementById('current-status-label');
            if (currentStatusLabel) currentStatusLabel.textContent = '';

            appendLog({
                level: data.success ? 'Success' : 'Error',
                message: data.msg,
                timestamp: Date.now()
            });
        });

        listen('translation-status', (event) => {
            const statusKey = event.payload; // "status_finished" 等
            const currentStatusLabel = document.getElementById('current-status-label');
            if (currentStatusLabel && state.currentLabels[statusKey]) {
                currentStatusLabel.textContent = state.currentLabels[statusKey];
            }
        });

        listen('translation-batch-update', (event) => {
            const data = event.payload; // { batch_index: x, total_batches: y, text: "..." }
            const batchProgress = document.getElementById('batch-progress-bar');
            const batchText = document.getElementById('batch-status-text');
            // 始終顯示，不再動態切換 display
            if (batchProgress && data.total_batches > 0) {
                const pct = (data.batch_index / data.total_batches) * 100;
                batchProgress.style.width = `${pct}%`;
                if (batchProgress.nextElementSibling) {
                    batchProgress.nextElementSibling.style.animation = 'pulse 1.5s infinite';
                }
            }
            if (batchText) {
                const mask = state.currentLabels.status_batch_mask;
                batchText.textContent = mask.replace('{}', data.batch_index).replace('{}', data.total_batches);
            }
        });

        listen('translation-log', (event) => {
            appendLog(event.payload);
        });
    }
}

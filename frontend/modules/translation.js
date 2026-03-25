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


                await invoke('start_translation', { 
                    config: state.currentConfig,
                    inputPaths: [state.currentConfig.path] 
                });
                setRunningState(true);
                if (progressBar) {
                    progressBar.style.width = '0%';
                }
                const batchContainer = document.getElementById('batch-progress-container');
                if (batchContainer) {
                    batchContainer.style.display = 'block'; // 啟動時直接展開
                }
                const batchProgress = document.getElementById('batch-progress-bar');
                if (batchProgress) {
                    batchProgress.style.width = '0%';
                }
                if (statusText) statusText.textContent = state.currentLabels.status_trans_starting;
            } catch (e) {
                appendLog(state.currentLabels.status_trans_failed_mask.replace('{}', e));
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
                statusText.textContent = `${Math.round((data.current / data.total) * 100)}%`;
            }
            // 狀態訊息僅顯示在狀態列，不寫入日誌區域
        });

        listen('translation-finished', (event) => {
            const data = event.payload; // { success: bool, msg: "..." }
            setRunningState(false);
            if (progressBar) progressBar.style.width = '100%';
            
            // 結束時保持顯示，讓狀態維持 100% 完成
            if (statusText)
                statusText.textContent = data.success
                    ? state.currentLabels.status_finished
                    : state.currentLabels.status_failed_or_cancelled;
            appendLog(data.msg);
        });

        listen('translation-batch-update', (event) => {
            const data = event.payload; // { batch_index: x, total_batches: y, text: "..." }
            const batchProgress = document.getElementById('batch-progress-bar');
            const batchText = document.getElementById('batch-progress-text');
            const batchContainer = document.getElementById('batch-progress-container');

            if (batchContainer && data.total_batches > 0) {
                batchContainer.style.display = 'block'; // 啟動時展開
            }
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

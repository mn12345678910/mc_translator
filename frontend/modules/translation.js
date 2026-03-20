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

    const inputs = document.querySelectorAll('.control-panel input:not(#input-path), .control-panel select, .control-panel textarea');
    inputs.forEach(el => el.disabled = isRunning);
    
    if (btnTranslate && btnPause && btnStop && btnResume) {
        if (isRunning) {
            btnTranslate.style.display = 'none';
            btnPause.style.display = 'inline-block';
            btnStop.style.display = 'inline-block';
            btnPause.textContent = state.currentLabels.btn_pause || '⏸️ 暫停';
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
                return alert(state.currentLabels.status_input_path_empty || '請先選擇檔案或主目錄！');
            }
            if (outputDir && outputDir.value.trim() === '') {
                return alert(state.currentLabels.status_output_dir_empty || '請選擇輸出目錄！');
            }
            try {
                // 確保從 DOM 取出最新狀態
                state.currentConfig.path = inputPath ? inputPath.value : '';
                state.currentConfig.output_dir = outputDir ? outputDir.value : '';

                await invoke('start_translation', { config: state.currentConfig });
                setRunningState(true);
                if (progressBar) { progressBar.style.width = '0%'; progressBar.style.display = 'block'; }
                if (statusText) statusText.textContent = state.currentLabels.status_starting || '🚀 正在啟動...';
            } catch (e) {
                appendLog((state.currentLabels.status_trans_failed_mask || '❌ 翻譯失敗: {}').replace('{}', e));
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
                btnResume.textContent = state.currentLabels.btn_resume || '▶️ 繼續';
            }
            if (statusText) statusText.textContent = state.currentLabels.status_paused || '⏸️ 已暫停';
        });
    }

    if (btnResume) {
        btnResume.addEventListener('click', async () => {
            await invoke('resume_translation');
            if (btnPause && btnResume) {
                btnPause.style.display = 'inline-block';
                btnResume.style.display = 'none';
            }
            if (statusText) statusText.textContent = state.currentLabels.status_resumed || '▶️ 繼續執行中';
        });
    }

    if (btnStop) {
        btnStop.addEventListener('click', async () => {
            await invoke('stop_translation');
            if (statusText) statusText.textContent = state.currentLabels.status_stopping || '🛑 正在停止中...';
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
            if (statusText) {
                const mask = state.currentLabels.status_progress_mask || '⏳ 執行中 ({} / {}) - {}';
                statusText.textContent = mask.replace('{}', data.current).replace('{}', data.total).replace('{}', data.msg);
            }
            if (data.msg) appendLog(data.msg);
        });

        listen('translation-finished', (event) => {
            const data = event.payload; // { success: bool, msg: "..." }
            setRunningState(false);
            if (progressBar) progressBar.style.width = '100%';
            if (statusText) statusText.textContent = data.success ? (state.currentLabels.status_done || '🎉 翻譯完成！') : '❌ 翻譯失敗 / 中止';
            appendLog(data.msg);
        });

        listen('translation-batch-update', (event) => {
            const data = event.payload; // { batch_index: x, total_batches: y, text: "..." }
            const batchProgress = document.getElementById('batch-progress-bar');
            const batchText = document.getElementById('batch-progress-text');
            if (batchProgress && data.total_batches > 0) {
                const pct = (data.batch_index / data.total_batches) * 100;
                batchProgress.style.width = `${pct}%`;
                if (batchProgress.nextElementSibling) {
                     batchProgress.nextElementSibling.style.animation = 'pulse 1.5s infinite';
                }
            }
            if (batchText) {
                const mask = state.currentLabels.status_batch_mask || '📦 批次進度 ({} / {})';
                batchText.textContent = mask.replace('{}', data.batch_index).replace('{}', data.total_batches);
            }
        });

        listen('native-log', (event) => {
             appendLog(event.payload);
        });
    }
}

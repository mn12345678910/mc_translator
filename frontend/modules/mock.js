// frontend/modules/mock.js
import { state } from './state.js';

export const allMockCommands = [
    'get_config',
    'get_default_config',
    'save_config',
    'get_style_config',
    'get_default_style_config',
    'save_style_config',
    'save_api_key_cmd',
    'get_api_key_cmd',
    'get_available_langs',
    'get_i18n_labels',
    'show_window',
    'get_models_from_provider',
    'start_translation',
    'pause_translation',
    'resume_translation',
    'stop_translation',
    'update_active_job_config',
    'query_dictionary',
    'edit_dictionary_item',
    'clear_user_dictionary',
    'import_user_dictionary',
    'export_user_dictionary',
    'open_dict_window',
    'open_dictionary_location',
    'open_path_dialog',
    'open_folder',
];

export async function initMockTools() {
    // 1. Inject Tauri Mock if missing
    if (!window.__TAURI__) {
        window.__MOCK_HIT_LIST = new Set();
        window.__TAURI__ = {
            core: {
                invoke: async (cmd, args) => {
                    const hitSet = window.__MOCK_HIT_LIST || new Set();
                    hitSet.add(cmd);
                    window.__MOCK_HIT_LIST = hitSet;
                    console.warn(`[GLOBAL-MOCK] Invoke: ${cmd}`, args);

                    const mocks = {
                        get_config: state.currentConfig,
                        get_default_config: {
                            api_provider: '無',
                            batch_size: 150,
                            timeout: 60,
                            pack_format: 15,
                        },
                        get_style_config: state.currentStyle,
                        get_default_style_config: { theme: 'dark', dark_bg: [45, 45, 50] },
                        get_api_key_cmd: 'MOCK_API_KEY_777',
                        get_available_langs: ['zh_tw', 'zh_cn', 'en_us', 'ja_jp'],
                        get_i18n_labels: async ({ lang }) => {
                            return state.currentLabels;
                        },
                        get_models_from_provider: ['gemini-1.5-flash', 'gpt-4o'],
                        query_dictionary: [[], 1],
                        open_path_dialog: 'C:\\Mock\\Path',
                        open_folder: null,
                        save_config: null,
                        save_style_config: null,
                        show_window: null,
                        save_api_key_cmd: null,
                        edit_dictionary_item: null,
                        open_dict_window: null,
                        clear_user_dictionary: null,
                        import_user_dictionary: null,
                        export_user_dictionary: null,
                        open_dictionary_location: null,
                        start_translation: null,
                        pause_translation: null,
                        update_active_job_config: null,
                        resume_translation: null,
                        stop_translation: null,
                    };

                    const res = mocks[cmd];
                    const finalRes = typeof res === 'function' ? await res(args) : res;
                    if (window.__refreshMockUICoverage) window.__refreshMockUICoverage();
                    return finalRes !== undefined ? finalRes : null;
                },
            },
            event: { listen: async () => ({ unlisten: () => {} }), emit: () => {} },
        };
    }

    // 2. Inject CSS
    const style = document.createElement('style');
    style.id = 'debug-style-injection';
    style.innerHTML = `
        .debug-overlay {
            position: fixed;
            top: 60px; /* Offset from header */
            right: 10px;
            background: rgba(0, 0, 0, 0.85);
            color: #0f0;
            padding: 10px;
            border-radius: 8px;
            z-index: 10000;
            font-family: 'JetBrains Mono', monospace;
            font-size: 11px;
            pointer-events: none;
            border: 1px solid rgba(0, 255, 0, 0.3);
            box-shadow: 0 4px 15px rgba(0,0,0,0.5);
            backdrop-filter: blur(4px);
        }
        .debug-list div {
            margin: 2px 0;
            white-space: nowrap;
        }
        .mock-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
            gap: 4px;
            font-size: 10px;
            font-family: monospace;
            max-height: 250px;
            overflow-y: auto;
            padding: 8px;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 6px;
            border: 1px solid rgba(255,255,255,0.05);
        }
        .dev-section {
            margin-top: 15px;
            padding-top: 10px;
            border-top: 1px solid rgba(255,255,255,0.1);
        }
        .header-sm {
            font-size: 12px;
            font-weight: bold;
            margin-bottom: 8px;
            opacity: 0.8;
            color: var(--accent-light, #aaf);
        }
        .debug-info-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 5px;
            margin-top: 10px;
            font-size: 11px;
            opacity: 0.7;
        }
    `;
    document.head.appendChild(style);

    // 3. Inject Debug UI
    const container = document.getElementById('debug-tools-container');
    if (container) {
        container.innerHTML = `
            <div class="dev-section">
                <div class="header-sm">Performance Stress Test</div>
                <div class="btn-group" style="gap: 10px; display: flex;">
                    <button class="btn btn-secondary" id="btn-stress-10k">Log x 10,000</button>
                    <button class="btn btn-secondary" id="btn-stress-1m">Log x 1,000,000</button>
                </div>
                <div class="debug-info-grid">
                    <div>Rendered: <span id="debug-rendered-count">0</span></div>
                    <div>Locked: <span id="debug-scroll-locked">False</span></div>
                    <div>Total Logs: <span id="debug-total-logs">0</span></div>
                    <div>Memory: <span id="debug-memory-est">~0 MB</span></div>
                </div>
            </div>
            <div class="dev-section">
                <div class="header-sm">Command Coverage (<span id="mock-coverage-count">0</span>)</div>
                <div id="mock-coverage-list" class="mock-grid"></div>
            </div>
        `;

        // Inject Overlay
        const overlay = document.createElement('div');
        overlay.id = 'mock-coverage-panel';
        overlay.className = 'debug-overlay';
        overlay.style.display = 'none';
        overlay.innerHTML = `
            <div class="debug-header" style="font-weight:bold; border-bottom: 1px solid #0f0; margin-bottom: 5px; padding-bottom: 2px;">
                MOCK COVERAGE: <span id="mock-coverage-percent">0%</span>
            </div>
            <div id="mock-hit-list" class="debug-list"></div>
        `;
        document.body.appendChild(overlay);

        // Bind visibility
        const chk = document.getElementById('chk-debug-tools');
        const syncVisibility = () => {
            const isVisible = state.currentConfig.show_debug_tools;
            container.style.display = isVisible ? 'block' : 'none';
            overlay.style.display = isVisible ? 'block' : 'none';
        };
        syncVisibility();
        if (chk) chk.addEventListener('change', syncVisibility);

        // Functional bindings
        setupStressTest();
        setupCoverageTracker();
    }
}

function setupStressTest() {
    async function stressTest(count) {
        console.log(`Starting stress test: ${count} logs`);
        const batchSize = 5000;
        let processed = 0;

        const generate = () => {
            const end = Math.min(processed + batchSize, count);
            for (let i = processed; i < end; i++) {
                if (window.__logViewer) {
                    window.__logViewer.appendLog(
                        `[Stress Test] This is log entry #${i + 1} for high-performance virtualization testing.`,
                        i % 10 === 0 ? 'warn' : i % 25 === 0 ? 'error' : 'info',
                        new Date().toLocaleTimeString()
                    );
                }
            }
            processed = end;
            if (processed < count) {
                window.requestAnimationFrame(generate);
            } else {
                console.log('Stress test completed.');
            }
        };
        generate();
    }

    const btn10k = document.getElementById('btn-stress-10k');
    const btn1m = document.getElementById('btn-stress-1m');
    if (btn10k) btn10k.addEventListener('click', () => stressTest(10000));
    if (btn1m) btn1m.addEventListener('click', () => stressTest(1000000));
}

function setupCoverageTracker() {
    window.__refreshMockUICoverage = () => {
        const listEl = document.getElementById('mock-coverage-list');
        const percentEl = document.getElementById('mock-coverage-percent');
        const countEl = document.getElementById('mock-coverage-count');
        const overlayListEl = document.getElementById('mock-hit-list');

        const hitSet = window.__MOCK_HIT_LIST || new Set();
        let hitCount = 0;

        if (listEl) {
            listEl.innerHTML = allMockCommands
                .map((cmd) => {
                    const isHit = hitSet.has(cmd);
                    if (isHit) hitCount++;
                    return `<div style="color: ${isHit ? '#4caf50' : '#888'}">
                    ${isHit ? '✅' : '⚠️'} ${cmd}
                </div>`;
                })
                .join('');
        }

        if (percentEl) {
            percentEl.textContent = `${Math.round((hitCount / allMockCommands.length) * 100)}%`;
        }

        if (countEl) countEl.innerText = hitSet.size;
        if (overlayListEl) {
            overlayListEl.innerHTML = Array.from(hitSet)
                .map((cmd) => `<div>• ${cmd}</div>`)
                .join('');
        }
    };

    window.__refreshMockUICoverage();
}

import { vi, beforeAll, beforeEach } from 'vitest';

// 1. 全域 Mock Tauri API 模組 (ESM)
vi.mock('@tauri-apps/api/tauri', () => ({
    invoke: vi.fn((cmd, args) => {
        if (globalThis.mockInvoke) return globalThis.mockInvoke(cmd, args);
        return Promise.resolve();
    }),
}));

beforeAll(() => {
    // 1. 全域 Mock Tauri API
    const mockInvoke = vi.fn();
    const mockListen = vi.fn(() => Promise.resolve(() => {}));
    globalThis.window = {
        __TAURI__: {
            core: { invoke: mockInvoke },
            event: { listen: mockListen }
        }
    };
    globalThis.mockInvoke = mockInvoke;
    globalThis.mockListen = mockListen;

    // 2. 模擬視窗原生對話框
    window.confirm = vi.fn();
    window.alert = vi.fn();
});

beforeEach(() => {
    if (globalThis.mockInvoke) globalThis.mockInvoke.mockReset();
    if (window.confirm) window.confirm.mockReset().mockReturnValue(true);
    if (window.alert) window.alert.mockReset();

    // 清空 DOM 避免測試交叉污染
    document.body.innerHTML = '';
});

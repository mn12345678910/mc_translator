import { vi, beforeAll, beforeEach } from 'vitest';

beforeAll(() => {
    // 1. 全域 Mock Tauri API
    const mockInvoke = vi.fn();
    globalThis.window = {
        __TAURI__: {
            core: { invoke: mockInvoke }
        }
    };
    globalThis.mockInvoke = mockInvoke; // 便於測驗內部引用驗證

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

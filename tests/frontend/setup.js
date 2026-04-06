import { vi, beforeAll, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/tauri', () => ({
    invoke: vi.fn((cmd, args) => {
        if (globalThis.mockInvoke) return globalThis.mockInvoke(cmd, args);
        return Promise.resolve();
    }),
}));

beforeAll(() => {
    const mockInvoke = vi.fn();
    const mockListen = vi.fn(() => Promise.resolve(() => {}));
    globalThis.window = {
        __TAURI__: {
            core: { invoke: mockInvoke },
            event: { listen: mockListen },
        },
    };
    globalThis.mockInvoke = mockInvoke;
    globalThis.mockListen = mockListen;

    window.confirm = vi.fn();
    window.alert = vi.fn();
});

beforeEach(() => {
    if (globalThis.mockInvoke) globalThis.mockInvoke.mockReset();
    if (window.confirm) window.confirm.mockReset().mockReturnValue(true);
    if (window.alert) window.alert.mockReset();

    document.body.innerHTML = '';
});

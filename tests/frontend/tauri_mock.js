export const invoke = (...args) => {
    if (globalThis.window && globalThis.window.__TAURI__ && globalThis.window.__TAURI__.core) {
        return globalThis.window.__TAURI__.core.invoke(...args);
    }
    if (globalThis.mockInvoke) return globalThis.mockInvoke(...args);
    return Promise.resolve();
};

export const event = {
    listen: (...args) => {
        if (globalThis.window && globalThis.window.__TAURI__ && globalThis.window.__TAURI__.event) {
            return globalThis.window.__TAURI__.event.listen(...args);
        }
        return Promise.resolve(() => {});
    }
};

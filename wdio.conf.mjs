export const config = {
    runner: 'local',
    specs: [
        './tests/e2e/**/*.test.js'
    ],
    exclude: [],
    maxInstances: 1,
    capabilities: [{
        maxInstances: 1,
        browserName: 'wry', // tauri 核心渲染視窗代表
        'tauri:options': {
            application: process.env.TAURI_BINARY_PATH || './target/debug/app'
        }
    }],
    logLevel: 'info',
    bail: 0,
    baseUrl: 'http://localhost',
    waitforTimeout: 10000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    services: [
        ['tauri', { autoInstallTauriDriver: true }]
    ],
    framework: 'mocha',
    reporters: ['spec'],
    mochaOpts: {
        ui: 'bdd',
        timeout: 60000
    },
};

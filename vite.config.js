import { defineConfig } from 'vite';

export default defineConfig({
  // 前端位於專案根目錄下的子目錄
  root: './frontend',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'esnext',
  },
  server: {
    port: 5173,
    strictPort: true,
  },
});

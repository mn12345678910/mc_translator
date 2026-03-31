import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  // 前端位於專案根目錄下的子目錄
  root: './frontend',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'esnext',
    rollupOptions: {
      input: {
        main: 'index.html', // 既然 root 是 frontend，此路徑相對於 root
        dict: 'dict.html',
      },
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  optimizeDeps: {
    include: ['@chenglou/pretext'],
  },
});

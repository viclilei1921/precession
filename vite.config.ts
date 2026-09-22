import path from 'node:path';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;
// 真机走 adb reverse，只转发 devUrl 的 1420。热更新若单独占用 1421，手机连不上。
const hmrOnDevPort = host === '127.0.0.1' || host === 'localhost' || host === '::1';

export default defineConfig(() => ({
  plugins: [react()],
  publicDir: 'public',
  css: {
    modules: {
      localsConvention: 'camelCase' as const
    }
  },
  resolve: {
    alias: {
      '@': path.resolve(import.meta.dirname, 'src')
    }
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: '0.0.0.0',
    hmr: host
      ? {
          protocol: 'ws',
          host,
          ...(hmrOnDevPort ? {} : { port: 1421 })
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
}));

/// <reference types="vitest" />
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import environment from 'vite-plugin-environment';
import dotenv from 'dotenv';

dotenv.config();

export default defineConfig({
  root: __dirname,
  build: {
    outDir: 'dist/',
    emptyOutDir: true,
    // increase warning threshold and add manual chunking to keep large deps separated
    chunkSizeWarningLimit: 1024, // 1MB
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            if (id.includes('@mui') || id.includes('material-ui')) {
              return 'mui';
            }
            if (id.includes('react') || id.includes('react-dom')) {
              return 'react-vendor';
            }
            return 'vendor';
          }
        },
      },
    },
  },
  optimizeDeps: {
    esbuildOptions: {
      define: {
        global: 'globalThis',
      },
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:4943',
        changeOrigin: true,
      },
    },
    allowedHosts: [
      '3000-crnlicp-chronolockreact-24zvh6kkm0j.ws-eu118.gitpod.io',
    ],
  },
  plugins: [
    react(),
    environment('all', { prefix: 'CANISTER_' }),
    environment('all', { prefix: 'DFX_' }),
  ],
  test: {
    environment: 'jsdom',
    setupFiles: 'setupTests.ts',
    globals: true,
  },
});

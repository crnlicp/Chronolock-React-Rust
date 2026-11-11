/// <reference types="vitest" />
import react from '@vitejs/plugin-react';
import { icpBindgen } from '@icp-sdk/bindgen/plugins/vite';
import { execSync } from 'node:child_process';
import { defineConfig, type ServerOptions } from 'vite';

// Canisters the frontend talks to (also the names in icp.yaml).
const CANISTERS = ['backend'];
const environment = process.env.ICP_ENVIRONMENT || 'local';

function canisterId(name: string): string | undefined {
  try {
    return execSync(`icp canister status ${name} -e ${environment} --id-only`, {
      encoding: 'utf-8',
      stdio: 'pipe',
    }).trim();
  } catch {
    return undefined;
  }
}

/** Simulates the `ic_env` cookie the frontend canister sets in production. */
function devServerConfig(): ServerOptions {
  const status = JSON.parse(
    execSync(`icp network status -e ${environment} --json`, { encoding: 'utf-8' }),
  );
  const params = CANISTERS.map((n) => [n, canisterId(n)] as const)
    .filter(([, id]) => id)
    .map(([n, id]) => `PUBLIC_CANISTER_ID:${n}=${id}`)
    .join('&');
  return {
    port: 3000,
    headers: {
      'Set-Cookie': `ic_env=${encodeURIComponent(
        `${params}&ic_root_key=${status.root_key}`,
      )}; SameSite=Lax;`,
    },
    proxy: {
      '/api': { target: status.api_url, changeOrigin: true },
    },
  };
}

export default defineConfig(({ command, mode }) => ({
  root: __dirname,
  build: {
    outDir: 'dist/',
    emptyOutDir: true,
    chunkSizeWarningLimit: 1024,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules')) {
            if (id.includes('@mui')) return 'mui';
            if (id.includes('@icp-sdk')) return 'icp';
            return 'vendor';
          }
        },
      },
    },
  },
  plugins: [
    react(),
    icpBindgen({ didFile: '../backend/backend.did', outDir: './src/bindings' }),
    icpBindgen({ didFile: './candid/icrc_ledger.did', outDir: './src/bindings' }),
  ],
  ...(command === 'serve' && mode !== 'test' ? { server: devServerConfig() } : {}),
  test: {
    environment: 'jsdom',
    setupFiles: 'setupTests.ts',
    globals: true,
  },
}));

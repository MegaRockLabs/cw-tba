import { resolve } from 'path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  root: '.',
  esbuild: { tsconfigRaw: '{}' },
  test: { clearMocks: true, globals: true, testTimeout: 185000, setupFiles: ['dotenv/config'] },
  resolve: { alias: [{ find: '~', replacement: resolve(__dirname, 'tests') }] },
});

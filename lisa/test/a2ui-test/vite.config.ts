import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    outDir: 'dist',
    target: 'es2022',
  },
  server: {
    proxy: {
      '/ws': {
        target: 'ws://127.0.0.1:42617',
        ws: true,
        rewrite: (path) => '/app',
      },
    },
  },
});

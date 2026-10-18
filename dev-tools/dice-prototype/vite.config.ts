import { defineConfig } from "vite";
export default defineConfig({
  resolve: {
    dedupe: ['react', 'react-dom'],
  },
  build: { rollupOptions: { input: { main: 'index.html', renderer: 'renderer.html', production: 'production.html' } } },
  server: {
    proxy: {
      "/roll": "http://127.0.0.1:8821",
      "/health": "http://127.0.0.1:8821",
    },
  },
});

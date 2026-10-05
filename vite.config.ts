import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Le port est fige parce que tauri.conf.json le designe nommement.
// Et `strictPort` evite le piege classique : si 1420 est pris, Vite
// glisserait en 1421 et Tauri ouvrirait une fenetre blanche sans rien dire.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**', '**/crates/**'] }
  },
  build: {
    target: 'chrome105',
    sourcemap: process.env.TAURI_ENV_DEBUG === 'true'
  }
});

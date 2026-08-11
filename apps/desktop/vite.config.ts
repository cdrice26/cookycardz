import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import tailwindcss from '@tailwindcss/vite';
import fs from 'fs';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],

  define: {
    __LICENSES_JS__: JSON.stringify(
      fs.readFileSync('src/licenses-js.txt', 'utf-8')
    ),
    __LICENSES_RUST__: JSON.stringify(
      fs.readFileSync('src/licenses-rust.txt', 'utf-8')
    )
  },

  server: { watch: { ignored: ['**/src-tauri/target/**', '**/target/**'] } },

  resolve: {
    alias: {
      // Ensure the desktop build resolves to the single React/react-dom copy
      // located at the workspace root. This prevents multiple React copies
      // (which cause invalid hook call errors) when running the Tauri app.
      react: path.resolve(import.meta.dirname, '../../node_modules/react'),
      'react-dom': path.resolve(import.meta.dirname, '../../node_modules/react-dom'),
      // JSX runtimes
      'react/jsx-runtime': path.resolve(
        import.meta.dirname,
        '../../node_modules/react/jsx-runtime'
      ),
      'react/jsx-dev-runtime': path.resolve(
        import.meta.dirname,
        '../../node_modules/react/jsx-dev-runtime'
      ),
      // react-dom client entry
      'react-dom/client': path.resolve(
        import.meta.dirname,
        '../../node_modules/react-dom/client'
      )
    }
  }
});

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import { dynamicBase } from 'vite-plugin-dynamic-base'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), dynamicBase({
    publicPath: 'window.__dynamic_base__',
    transformIndexHtml: true
  })],
  base: "/__dynamic_base__/configure"
})

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import mkcert from 'vite-plugin-mkcert'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), mkcert()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    host: '0.0.0.0',
    https: true,
    proxy: {
      '/api': {
        target: process.env.NODE_ENV === 'development' && process.env.DOCKER_ENV
          ? 'https://backend:6000'
          : 'https://localhost:6000',
        changeOrigin: true,
        secure: false,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
      '/uploads': {
        target: process.env.NODE_ENV === 'development' && process.env.DOCKER_ENV
          ? 'https://backend:6000'
          : 'https://localhost:6000',
        changeOrigin: true,
        secure: false,
      },
      '/ws': {
        target: process.env.NODE_ENV === 'development' && process.env.DOCKER_ENV
          ? 'wss://backend:6000'
          : 'wss://localhost:6000',
        ws: true,
        secure: false,
      },
    },
  },
})

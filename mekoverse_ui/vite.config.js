import { defineConfig } from 'vite'
import basicSsl from '@vitejs/plugin-basic-ssl'

export default defineConfig({
  plugins: [
    basicSsl()
  ],
  server: {
    https: true,
    port: 5173,
    // 必要に応じてCORSも許可しておく
    cors: true 
  }
})
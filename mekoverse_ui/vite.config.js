import { defineConfig } from 'vite'
import mkcert from 'vite-plugin-mkcert'

export default defineConfig({
  plugins: [
    mkcert()
  ],
  server: {
    https: true,
    port: 5173,
    // 必要に応じてCORSも許可しておく
    cors: true 
  },
  build: {
    lib: {
      // エントリポイント（initGame を export しているファイル）
      entry: 'src/main.ts', 
      name: 'MekouApp',
      // ESモジュール形式を指定
      formats: ['es'],
      // 出力ファイル名を固定（ハッシュを付けない方がデバッグしやすい）
      fileName: 'index'
    },
    rollupOptions: {
      // エンジン側で用意している共通ライブラリを二重に含めない設定
      external: ['@mekou/engine-api'],
    }
  }
})
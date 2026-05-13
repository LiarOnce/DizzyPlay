import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { execSync } from "child_process";
import pkg from "./package.json" assert { type: "json" };

const host = process.env.TAURI_DEV_HOST;

const gitCommit = execSync("git rev-parse --short HEAD").toString().trim();
const appVersion = `${pkg.version}+${gitCommit}`;

export default defineConfig(async () => ({
  plugins: [vue()],
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/docs/**"],
    },
    // 代理 /apis/ 请求到 dizzylab.net，解决浏览器开发时的 CORS 问题
    proxy: {
      "/apis": {
        target: "https://www.dizzylab.net",
        changeOrigin: true,
        secure: false,
        headers: {
          Referer: "https://www.dizzylab.net",
        },
      },
      // 代理 cdn 图片请求，添加 Referer 头解决防盗链
      "/cdn": {
        target: "https://cdn.dizzylab.net",
        changeOrigin: true,
        secure: false,
        headers: {
          Referer: "https://www.dizzylab.net",
        },
        rewrite: (path) => path.replace(/^\/cdn/, ""),
      },
      // 代理 streaming 音频请求，解决浏览器开发时的 CORS 问题
      "/streaming": {
        target: "https://streaming.dizzylab.net",
        changeOrigin: true,
        secure: false,
        headers: {
          Referer: "https://www.dizzylab.net",
        },
        rewrite: (path) => path.replace(/^\/streaming/, ""),
      },
    },
  },
}));

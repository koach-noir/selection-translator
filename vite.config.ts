import { defineConfig, type Plugin } from "vite";

const host = process.env.TAURI_DEV_HOST;

// devサーバーで /popup/ → /src/popup/ にリライト
function multiPageRewrite(): Plugin {
  return {
    name: "multi-page-rewrite",
    configureServer(server) {
      server.middlewares.use((req, _res, next) => {
        if (req.url?.startsWith("/popup/")) {
          req.url = "/src" + req.url;
        } else if (req.url?.startsWith("/settings/")) {
          req.url = "/src" + req.url;
        }
        next();
      });
    },
  };
}

export default defineConfig({
  clearScreen: false,
  plugins: [multiPageRewrite()],
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
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        popup: "src/popup/index.html",
        settings: "src/settings/index.html",
      },
    },
  },
});

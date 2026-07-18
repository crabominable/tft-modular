import { defineConfig } from "vite";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root: __dirname,
  publicDir: "public",
  server: {
    fs: { allow: ["../.."] },
  },
  resolve: {
    alias: {
      "@mod": path.resolve(__dirname, "../../mods/reference-mod"),
    },
  },
  optimizeDeps: {
    exclude: ["@tft/plugin-loader", "@tft/plugin-schema"],
  },
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
});

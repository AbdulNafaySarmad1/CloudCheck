import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import packageManifest from "./package.json";

export default defineConfig({
  plugins: [react()],
  define: { __APP_VERSION__: JSON.stringify(packageManifest.version) },
  clearScreen: false,
  build: { outDir: "frontend-dist" },
  server: { port: 1420, strictPort: true },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: "./src/test/setup.ts",
  },
});

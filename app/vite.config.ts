/**
 * Vite config for the webview bundle. Tauri loads `dist` in production and
 * the dev server on 127.0.0.1:1420 during `tauri dev`. Vitest reads the same
 * file and runs every `.test.ts` and `.test.tsx` file under `src` in a
 * happy-dom page.
 */
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

/** Dev server port, matching `devUrl` in tauri.conf.json. */
const DEV_PORT = 1420;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { host: "127.0.0.1", port: DEV_PORT, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true, target: ["es2022", "safari15", "chrome110"], sourcemap: false },
  test: { environment: "happy-dom", include: ["src/**/*.test.{ts,tsx}"] },
});

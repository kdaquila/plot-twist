/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  root: "ui",
  plugins: [react()],
  // Keep the dev-server cache in the root node_modules/ rather than ui/node_modules/.
  cacheDir: "../node_modules/.vite",
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: { outDir: "../target/ui", emptyOutDir: true, target: "es2022" },
  test: { environment: "node" },
});

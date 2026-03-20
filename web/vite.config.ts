import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "/kessel-sabacc/",
  resolve: {
    tsconfigPaths: true,
  },
  optimizeDeps: {
    exclude: ["sabacc-wasm"],
  },
});

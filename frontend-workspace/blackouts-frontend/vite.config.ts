import { defineConfig, splitVendorChunkPlugin } from "vite";
import react from "@vitejs/plugin-react";

console.log(process.env.VITE_API);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), splitVendorChunkPlugin()],
  server: {
    proxy: {
      "/api": `127.0.0.1:8080`,
    },
  },
});

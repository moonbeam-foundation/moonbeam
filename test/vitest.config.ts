import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  resolve: {
    alias: {
      helpers: path.resolve(__dirname, "helpers"),
    },
  },
});

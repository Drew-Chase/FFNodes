import {defineConfig} from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    plugins: [react()],
    esbuild: {
        legalComments: "none",
        supported: {
            "top-level-await": true // browsers can handle top-level-await features
        }
    },
    clearScreen: false,
    server: {
        host: true,
        port: 5173,
        strictPort: true,
        hmr: {
            protocol: "ws",
            host: "localhost",
            port: 5173,
            clientPort: 5173,
            overlay: true
        },
        watch: {
            ignored: ["**/src-*/**"]
        }
    },
    build: {
        outDir: "../target/wwwroot"
    }
});
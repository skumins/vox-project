import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { viteStaticCopy } from "vite-plugin-static-copy";

export default defineConfig({
    optimizeDeps: {
        include: ["@ricky0123/vad-react", "@ricky0123/vad-web", "onnxruntime-web"],
    },

    plugins: [
        react(),
        viteStaticCopy({
            targets: [
                {
                    src: "node_modules/@ricky0123/vad-web/dist/vad.worklet.bundle.min.js",
                    dest: "./",
                },
            ],
        }),
    ],

    server: {
        port: 5173,
        headers: {
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Embedder-Policy": "require-corp",
        },
        proxy: {
            "/ws": { target: "ws://localhost:3000", ws: true, rewriteWsOrigin: true },
            "/transcribe": { target: "http://localhost:3000" },
        },
    },
});
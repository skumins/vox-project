import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { viteStaticCopy } from "vite-plugin-static-copy";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
    optimizeDeps: {
        include: ["@ricky0123/vad-react", "@ricky0123/vad-web", "onnxruntime-web"],
    },

    plugins: [
        react(),
        {
            name: "serve-ort-mjs",
            configureServer(server) {
                server.middlewares.use((req, res, next) => {
                    const url = req.url ?? "";
                    if (url.includes("ort-wasm") && url.includes(".mjs")) {
                        const filename = url.split("/").pop()!.split("?")[0];
                        const filePath = path.resolve(
                            __dirname,
                            "node_modules/onnxruntime-web/dist",
                            filename
                        );
                        if (fs.existsSync(filePath)) {
                            res.setHeader("Content-Type", "application/javascript");
                            res.setHeader("Cross-Origin-Resource-Policy", "cross-origin");
                            fs.createReadStream(filePath).pipe(res);
                            return;
                        }
                    }
                    next();
                });
            },
        },

        viteStaticCopy({
            targets: [
                {
                    src: "node_modules/@ricky0123/vad-web/dist/vad.worklet.bundle.min.js",
                    dest: "./",
                },
                {
                    src: "node_modules/@ricky0123/vad-web/dist/silero_vad_v5.onnx",
                    dest: "./",
                },
                {
                    src: "node_modules/@ricky0123/vad-web/dist/silero_vad_legacy.onnx",
                    dest: "./",
                },
                {
                    src: "node_modules/onnxruntime-web/dist/*.wasm",
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
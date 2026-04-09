import * as ort from "onnxruntime-web";
ort.env.wasm.numThreads = 1;
ort.env.wasm.wasmPaths = "/";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./App.css";
import App from "./App";

const rootElement = document.getElementById("root");

createRoot(rootElement!).render(
    <StrictMode>
        <App />
    </StrictMode>
);
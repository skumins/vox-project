import * as ort from "onnxruntime-web";
ort.env.wasm.wasmPaths =
  "https://cdn.jsdelivr.net/npm/onnxruntime-web@1.24.3/dist/";

import { createRoot } from "react-dom/client";
import "./App.css";
import App from "./App";

createRoot(document.getElementById("root")!).render(<App />);

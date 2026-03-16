import { useRef, useCallback } from "react";

interface SocketCallbacks {
    onTranscript: (text: string) => void;
    onSummary: (text: string) => void;
    onStatus: (msg: string, isError: boolean) => void;
}

export function useVoxaSocket(callbacks: SocketCallbacks) {
    // WebSocket storage
    const wsRef = useRef<WebSocket | null>(null);

    const callbacksRef = useRef(callbacks);
    callbacksRef.current = callbacks;

    // connection
    const connect = useCallback((): Promise<void> => {
        return new Promise((resolve, reject) => {
            const ws = new WebSocket("ws://localhost:5173/ws");
            
            ws.onopen = () => {
                wsRef.current = ws;
                callbacksRef.current.onStatus("Connected", false);
                resolve();
            };

            ws.onerror = () => reject(new Error("WebSocket connection failed"));

            ws.onclose = () => {
                wsRef.current = null;
                callbacksRef.current.onStatus("Disconnected", false);
            };

            // Processing incoming messages
            ws.onmessage = (event: MessageEvent<string>) => {
                const data = event.data;
                if (data.startsWith("transcript:")) {
                    callbacksRef.current.onTranscript(data.slice("transcript:".length));
                } else if (data.startsWith("summary")) {
                    callbacksRef.current.onSummary(data.slice("summary:".length));
                } else if (data.startsWith("status:")) {
                    callbacksRef.current.onStatus(data.slice("status:".length), false);
                } else if (data.startsWith("error:")) {
                    callbacksRef.current.onStatus(data.slice("error:".length), true);
                }
            };
        });
    }, []);


    // sending
    const sendAudio = useCallback((buffer: ArrayBuffer) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(buffer);
        }
    }, []);

    const sendCommand = useCallback((cmd: "summarize" | "stop") => {
        wsRef.current?.send(cmd);
    }, []);

    return { connect, sendAudio, sendCommand };
}
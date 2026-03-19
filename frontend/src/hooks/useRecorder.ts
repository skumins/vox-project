import { useRef, useCallback } from "react";

export function useRecorder(onChunk: (buffer: ArrayBuffer) => void) {
    const streamRef = useRef<MediaStream | null>(null);
    const recorderRef = useRef<MediaRecorder | null>(null);

    const start = useCallback(async (): Promise<void> => {
        streamRef.current = await navigator.mediaDevices.getUserMedia({
            audio: {
                echoCancellation: true,
                noiseSuppression: true,
            },
        });

        const mimeType = MediaRecorder.isTypeSupported("audio/webm;codecs=opus")
            ? "audio/webm;codecs=opus" // Chrome, Firefox — opus compresses voice better
            : "audio/webm";  // Safari fallback

        const recorder = new MediaRecorder(streamRef.current, { mimeType });

        recorder.ondataavailable = async (event: BlobEvent) => {
            if (event.data.size > 0) {
                const buffer = await event.data.arrayBuffer();
                onChunk(buffer);
            }
        };
        
        recorder.start(1000);
        recorderRef.current = recorder;

    }, [onChunk]);

    const stop = useCallback(() => {
        if (recorderRef.current && recorderRef.current.state !== "inactive") {
            recorderRef.current.stop();
        }
        streamRef.current?.getTracks().forEach(track => track.stop());
        recorderRef.current = null;
        streamRef.current = null;
    }, []);

    return { start, stop };
}
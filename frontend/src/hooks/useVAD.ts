import { useCallback, useRef } from "react";
import { useMicVAD } from "@ricky0123/vad-react";

export function useVAD(onSegment: (buffer: ArrayBuffer) => void) {
    const isActiveRef = useRef(false);

    function float32ToInt16(float32: Float32Array): ArrayBuffer {
        const int16 = new Int16Array(float32.length);
        for (let i = 0; i < float32.length; i++) {
            const clamped = Math.max(-1, Math.min(1, float32[i]));
            int16[i] = Math.round(clamped * 32767);
        }
        return int16.buffer;
    }

    const vad = useMicVAD({
        startOnLoad: false,
        model: "v5",
        baseAssetPath: "https://cdn.jsdelivr.net/npm/@ricky0123/vad-web@0.0.30/dist/",
        onnxWASMBasePath: "https://cdn.jsdelivr.net/npm/onnxruntime-web@1.24.3/dist/",

        positiveSpeechThreshold: 0.5,
        negativeSpeechThreshold: 0.3,
        minSpeechMs: 300,
        preSpeechPadMs: 300,

        onSpeechEnd: (audio: Float32Array) => {
            console.log(`Speech segment: ${audio.length} samples = ${(audio.length / 16000).toFixed(2)}s`);
            if (!isActiveRef.current) return;
            onSegment(float32ToInt16(audio));
        },

        onVADMisfire: () => {
            console.log("VAD misfire ignored");
        },
    });

    const start = useCallback(async (): Promise<void> => {
        if (vad.loading) {
            throw new Error("VAD model is loading, please wait");
        }
        if (vad.errored) {
            console.error("VAD error details:", vad.errored);
            throw new Error("VAD failed to load");
        }
        isActiveRef.current = true;
        vad.start();
    }, [vad]);

    const stop = useCallback(() => {
        isActiveRef.current = false;
        vad.pause();
    }, [vad]);

    return {
        start,
        stop,
        isSpeaking: vad.userSpeaking,
        loading: vad.loading,
        errored: vad.errored,
    };
}
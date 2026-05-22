import { useCallback, useRef } from "react";
import { useMicVAD, utils } from "@ricky0123/vad-react";

export function useVAD(onSegment: (buffer: ArrayBuffer) => void) {
  const isActiveRef = useRef(false);

  const vad = useMicVAD({
    startOnLoad: false,
    model: "v5",
    baseAssetPath:
      "https://cdn.jsdelivr.net/npm/@ricky0123/vad-web@0.0.30/dist/",
    onnxWASMBasePath:
      "https://cdn.jsdelivr.net/npm/onnxruntime-web@1.24.3/dist/",

    positiveSpeechThreshold: 0.5,
    negativeSpeechThreshold: 0.35,
    minSpeechMs: 500,
    preSpeechPadMs: 300,
    redemptionMs: 2000,

    onSpeechEnd: (audio: Float32Array) => {
      if (!isActiveRef.current) return;
      const wavBuffer = utils.encodeWAV(audio);
      onSegment(wavBuffer);
    },

    onVADMisfire: () => {},
  });

  const start = useCallback(async (): Promise<void> => {
    if (vad.loading) throw new Error("VAD is loading, please wait");
    if (vad.errored) throw new Error("VAD failed to load");
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

import {useState, useCallback } from "react";
import { useVoxaSocket } from "./hooks/useVoxaSocket";
import { useRecorder } from "./hooks/useRecorder";

type AppStatus = "idle" | "recording" | "processing" | "error";

export default function App() {
    const [status, setStatus] = useState<AppStatus>("idle");
    const [statusText, setStatusText] = useState("Ready");
    const [transcript, setTranscript] = useState("");
    const [summary, setSummary] = useState("");

    const handleTranscript = useCallback((text: string) => {
        setTranscript(prev => prev + text + " ");
    }, []);

    const handleSummary = useCallback((text: string) => {
        setSummary(text);
        setStatus("idle");
        setStatusText("Done");
    }, []);

    const handleStatus = useCallback((msg: string, isError: boolean) => {
        setStatusText(msg);
        if(isError) setStatus("error");
    }, []);


    const socket = useVoxaSocket({
        onTranscript: handleTranscript,
        onSummary: handleSummary,
        onStatus: handleStatus,
    });

    const recorder = useRecorder(socket.sendAudio);


    async function handleStart() {
        try {
            setStatusText("Connecting...");
            await socket.connect();
            await recorder.start();
            setStatus("recording");
            setStatusText("Recording");
        } catch (err) {
            setStatusText(err instanceof Error ? err.message : "Unknow error");
        }
    }

    function handleStop() {
        recorder.stop();
        socket.sendCommand("stop");
        setStatus("idle");
        setStatusText("Stopped");
    }

    function handleSummarize() {
        socket.sendCommand("summarize");
        setStatus("processing");
        setStatusText("Processing...");
    }

    const isRecording = status === "recording";
    const isProcessing = status === "processing";
    const canSummarize = transcript.length > 0 && !isRecording && !isProcessing;


    return (
        <div className="app">
            <header>
                <h1>VOXA</h1>
                {}
                <span className={`status status-${status}`}>{statusText}</span>
            </header>

            <div className="controls">
                <button onClick={handleStart} disabled={isRecording || isProcessing}>
                    Start
                </button>

                <button onClick={handleStop} disabled={!isRecording}>
                    Stop
                </button>

                <button onClick={handleSummarize} disabled={!canSummarize}>
                    {isProcessing ? "Processing...": "Summarize"}
                </button>
            </div>

            <div className="panels">
                <section className="panel">
                    <h2>Transcript</h2>
                    {}
                    <textarea value={transcript} readOnly placeholder="Transcript will apper here..." />
                </section>

                <section className="panel">
                    <h2>Summary</h2>
                    <textarea value={summary} readOnly placeholder="Summary will apper here..." />
                </section>
            </div>
        </div>
    );

}
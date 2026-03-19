import {useState, useCallback, useEffect } from "react";
import { useVoxaSocket } from "./hooks/useVoxaSocket";
import { useRecorder } from "./hooks/useRecorder";

type AppStatus = "idle" | "recording" | "processing" | "error";

export default function App() {
    const [status, setStatus] = useState<AppStatus>("idle");
    const [statusText, setStatusText] = useState("Ready");
    const [transcript, setTranscript] = useState("");
    const [interim, setInterim] = useState("");
    const [summary, setSummary] = useState("");
    const [transcriptLang, setTranscriptLang] = useState("no");
    const [summaryLang, setSummaryLang] = useState("uk");

    const handleTranscript = useCallback((text: string, isFinal: boolean) => {
        if (isFinal) {
            setTranscript(prev => prev + text + " ");
            setInterim("");
        } else {
            setInterim(text);
        }

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

    useEffect(() => {
        if (summary !== "") {
            socket.sendCommand("disconnect");
        }
    }, [summary]);

    async function handleStart() {
        try {
            setTranscript("");
            setInterim("");
            setSummary("");
            setStatusText("Connecting...");
            await socket.connect({ lang: transcriptLang, summaryLang });
            await recorder.start();
            setStatus("recording");
            setStatusText("Recording");
        } catch (err) {
            setStatus("error");
            setStatusText(err instanceof Error ? err.message : "Unknow error");
        }
    }

    function handleStop() {
        recorder.stop();
        socket.sendCommand("stop");
        setStatus("idle");
        setStatusText("Stopped. Press Summarize.");
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

            <div className="lang-row">
                <label>
                    <span>Transcript</span>
                    <select
                        value={transcriptLang}
                        onChange={e => setTranscriptLang(e.target.value)}
                        disabled={isRecording}
                    >
                        <option value="no">Norwegian</option>
                        <option value="en">English</option>
                        <option value="uk">Ukrainian</option>
                        <option value="de">German</option>
                        <option value="multi">Auto-detect</option>
                    </select>
                </label>

                <label>
                    <span>Summary</span>
                    <select
                        value={summaryLang}
                        onChange={e => setSummaryLang(e.target.value)}
                        disabled={isProcessing}
                    >
                        <option value="en">English</option>
                        <option value="de">German</option>
                        <option value="no">Norwegian</option>
                        <option value="uk">Ukrainian</option>
                    </select>
                </label>
            </div>

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
                    <textarea value={transcript + (interim ? `[${interim}]` : "")} readOnly placeholder="Transcript will apper here..." />
                </section>
                <section className="panel">
                    <h2>Summary</h2>
                    <textarea value={summary} readOnly placeholder="Summary will apper here..." />
                </section>
            </div>
        </div>
    );

}
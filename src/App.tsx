import { h } from "preact";
import { useContext, useEffect, useState } from "preact/hooks";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { GlobalChannel } from "./globalChannel.ts";
export default function App() {
  const [payloadText, setPayloadText] = useState("");
  const [totalBytes, setTotalBytes] = useState(0);
  const [startTime, setStartTime] = useState(Date.now());

  const channel = useContext(GlobalChannel);

  useEffect(() => {
    if (channel.receiver.locked) {
      return;
    }
    const reader = channel.receiver.getReader();
    let reading = true;
    const readloop = (async () => {
      const decoder = new TextDecoder("utf-8");
      while (reading) {
        const { done, value } = await reader.read();
        if (done) {
          return;
        }
        setTotalBytes((t) => t + value.byteLength);
        setPayloadText(decoder.decode(value));
      }
    })();

    return () => {
      (async () => {
        reading = false;
        await readloop;
        reader.releaseLock();
      })();
    };
  });

  const elapsed = Date.now() - startTime;
  const mbps = (totalBytes * 8 / 1000000) / (elapsed / 1000);

  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      <h1>Welcome to Tauri!</h1>
      <button
        onClick={() => {
          invoke("start");
        }}
      >
        start
      </button>

      <textarea value={payloadText}></textarea>

      <div style={{ display: "flex", flexDirection: "row", gap: "1em" }}>
        <span>{totalBytes}B/{elapsed}ms</span>
        <span>{mbps.toFixed(3)}Mbps</span>
      </div>
    </div>
  );
}

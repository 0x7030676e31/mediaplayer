import { createSignal } from "solid-js";

let sse: EventSource | null = null;
const url = "http://localhost:7777/api/dashboard/stream";

type Payload
  = { type: "Ready", payload: { library: Media[], clients: Client[] } }
  | { type: "ClientCreated", payload: Client }
  | { type: "ClientConnected", payload: number }
  | { type: "ClientDisconnected", payload: number[] }
  | { type: "MediaDownloaded", payload: { media: number, client: number } };

function handle(payload: string) {
  const data = JSON.parse(payload) as Payload;
}

export function useStream() {
  const [ ready, setReady ] = createSignal(false);

  sse = new EventSource(url);
  sse.onopen = () => console.log("Connected to server");
  sse.onmessage = (e) => handle(e.data);
  sse.onerror = () => {
    console.log("Disconnected from server");
    setReady(false);
  };

  return { connected: ready };
}
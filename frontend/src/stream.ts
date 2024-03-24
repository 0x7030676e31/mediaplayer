import { batch, createSignal } from "solid-js";

let sse: EventSource | null = null;
export const base_url = import.meta.env.DEV ? "http://192.168.0.91:7777" : "";

type Payload
  = { type: "Ready", payload: { library: Media[], clients: Client[] } }
  | { type: "ClientCreated", payload: Client }
  | { type: "ClientConnected", payload: number }
  | { type: "ClientDisconnected", payload: number[] }
  | { type: "MediaCreated", payload: { id: number, name: string, length: number } }
  | { type: "MediaDeleted", payload: number }
  | { type: "MediaDownloaded", payload: { media: number, client: number } }
  | { type: "MediaStarted", payload: { media: number, client: number } }
  | { type: "MediaStopped", payload: number };

type TempMedia = {
  nonce: number;
  name: string;
}

type Log = {
  time: number;
  message: string;
}

const [ logs, setLogs ] = createSignal<Log[]>([]);
const [ media, setMedia ] = createSignal<Media[]>([]);
const [ tempMedia, setTempMedia ] = createSignal<TempMedia[]>([]);
const [ clients, setClients ] = createSignal<Client[]>([]);

export const T0 = Date.now();
export function useLogs() {
  return [ logs, setLogs ] as const;
}

export function useMedia() {
  return [ media, setMedia ] as const;
}

export function useTempMedia() {
  return [ tempMedia, setTempMedia ] as const;
}

export function useClients() {
  return [ clients, setClients ] as const;
}

function log(message: string) {
  setLogs([ { time: Date.now(), message }, ...logs() ]);
}

window.get_media = media;
window.get_clients = clients;

const ack: number[] = [];

function handle(message: string) {
  const payload = JSON.parse(message) as { payload: Payload, nonce: number | null, ack: number };
  if (ack.includes(payload.ack)) return;
  console.log(payload);

  setTimeout(() => ack.shift(), 1000 * 60 * 5);
  ack.push(payload.ack);
  
  const data = payload.payload;
  switch (data.type) {
    case "Ready":
      batch(() => {
        log(`Received initial data from server, received ${data.payload.library.length} media and ${data.payload.clients.length} clients`);

        setMedia(data.payload.library);
        setClients(data.payload.clients);
        
        callback?.();
      });
      break;

    case "MediaCreated":
      const temp_media = tempMedia().find(m => m.nonce === payload.nonce);
      batch(() => {
        log(`Media ${temp_media!.name} has been uploaded`);

        setMedia([...media(), { id: data.payload.id, name: temp_media!.name, length: data.payload.length, downloaded: [] }]);
        setTempMedia(tempMedia().filter(m => m.nonce !== payload.nonce));
      });
      break;

    case "MediaDeleted":
      batch(() => {
        log(`Media ${data.payload} has been deleted`);
        setMedia(media().filter(m => m.id !== data.payload));
      });
      break;

    case "MediaDownloaded":
      batch(() => {
        log(`Media ${data.payload.media} has been downloaded by client ${data.payload.client}`);
        setMedia(media().map(m => m.id === data.payload.media ? { ...m, downloaded: [ ...m.downloaded, data.payload.client ] } : m));
      });
      break;

    default:
      console.log("Unhandled event", data);
  }
}

let callback: () => void;
export function useStream() {
  const [ ready, setReady ] = createSignal(false);

  sse = new EventSource(base_url + "/api/dashboard/stream");
  sse.onopen = () => log("Connected to the server");
  sse.onmessage = (e) => handle(e.data);
  sse.onerror = () => {
    log("Disconnected from server");
    setReady(false);
  };

  callback = () => setReady(true);
  return { connected: ready };
}

function get_nonce(): number {
  return Math.floor(Math.random() * 1000000000);
}

export async function add_media(file: File) {
  const nonce = get_nonce();
  batch(() => {
    setTempMedia([...tempMedia(), { nonce, name: file.name }]);
    log(`Media ${file.name} has been set for upload`);
  });
    
  const name = encodeURIComponent(file.name);
  await fetch(`${base_url}/api/media/upload/${nonce}/${name}`, {
    method: "POST",
    body: file,
  });
}

export async function delete_media(id: number) {
  log(`Media ${id} has been set for deletion`);
  await fetch(`${base_url}/api/media/${id}`, { method: "DELETE" });
}

export async function request_download(id: number) {
  log(`Media ${id} has been requested for download`);
  const target = media().find(m => m.id === id);

  await fetch(`${base_url}/api/media/${id}/request_download`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(clients().filter(c => !target?.downloaded.includes(c.id)).map(c => c.id)),
  });
}

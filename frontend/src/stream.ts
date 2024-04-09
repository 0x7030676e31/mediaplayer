import { batch, createSignal } from "solid-js";

let sse: EventSource | null = null;
// export const base_url = import.meta.env.DEV ? "http://192.168.0.91:7777" : window.location.origin;
export const base_url = import.meta.env.DEV ? "http://localhost:7777" : window.location.origin + "/mp"
// export const base_url = import.meta.env.DEV ? "http://10.25.71.42:7777" : "";
// export const base_url = import.meta.env.DEV ? "http://localhost:7777" : "";


type Payload
  = { type: "Ready", payload: { library: Media[], clients: Client[], groups: Group[], playing: number[] } }
  | { type: "ClientCreated", payload: Client }
  | { type: "ClientConnected", payload: number }
  | { type: "ClientDisconnected", payload: number[] }
  | { type: "ClientDeleted", payload: number }
  | { type: "ClientRenamed", payload: [number, string | null] }
  | { type: "MediaCreated", payload: { id: number, name: string, length: number } }
  | { type: "MediaDeleted", payload: number }
  | { type: "MediaDownloaded", payload: { media: number, client: number } }
  | { type: "MediaStarted", payload: { media: number, client: number } }
  | { type: "MediaStopped", payload: number }
  | { type: "GroupCreated", payload: number }
  | { type: "GroupEdited", payload: Group }
  | { type: "GroupMemberAdded", payload: [number, number] }
  | { type: "GroupMemberDeleted", payload: [number, number] }
  | { type: "GroupDeleted", payload: number };


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
const [ groups, setGroups ] = createSignal<Group[]>([]);
const [ clients, setClients ] = createSignal<Client[]>([]);
const [ playing, setPlaying ] = createSignal<number[]>([]);

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

export function useGroups() {
  return [ groups, setGroups ] as const;
}

export function usePlaying() {
  return [ playing, setPlaying ] as const;
}

export function useClients() {
  return [ clients, setClients ] as const;
}

function log(message: string) {
  setLogs([ { time: Date.now(), message }, ...logs() ]);
}

window.get_media = media;
window.get_clients = clients;
window.get_groups = groups;

const ack: number[] = [];
const change_alias_nonce: number[] = [];

let activity_callback: ((activity: "Online" | "Offline", id: number) => void) | null = null;
export function onActivityChange(callback: (activity: "Online" | "Offline", id: number) => void) {
  activity_callback = callback;
  return () => activity_callback = null;
}

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
        log(`Received initial data from server, received ${data.payload.library.length} media, ${data.payload.clients.length} clients and ${data.payload.groups.length} groups`);

        batch(() => {
          setMedia(data.payload.library);
          setClients(data.payload.clients);
          setGroups(data.payload.groups.reverse());
          setPlaying(data.payload.playing);
        });
        
        callback?.(true);
      });
      break;

    case "ClientCreated":
      batch(() => {
        log(`Client ${data.payload.hostname} has been created`);
        setClients([...clients(), data.payload]);
      });
      break;

    case "ClientConnected":
      batch(() => {
        log(`Client ${data.payload} has connected`);
        setClients(clients().map(c => c.id === data.payload ? { ...c, activity: { activity: "Online" } as const } : c));
      });

      activity_callback?.("Online", data.payload);
      break;

    case "ClientDisconnected":
      batch(() => {
        log(`Client ${data.payload} has disconnected`);
        setClients(clients().map(c => data.payload.includes(c.id) ? { ...c, activity: { activity: "Offline", timestamp: Date.now() } as const } : c));
        setPlaying(playing().filter(p => !data.payload.includes(p)));
      });

      data.payload.forEach(id => activity_callback?.("Offline", id));
      break;

    case "ClientDeleted":
      batch(() => {
        log(`Client ${data.payload} has been deleted`);
        setClients(clients().filter(c => c.id !== data.payload));
        setPlaying(playing().filter(p => p !== data.payload));
        setMedia(media().map(m => ({ ...m, downloaded: m.downloaded.filter(d => d !== data.payload) })));
      });
      break;

    case "ClientRenamed":
      if (change_alias_nonce.includes(payload.nonce!)) return;
      batch(() => {
        log(`Client ${data.payload[0]} has been renamed to ${data.payload[1]}`);
        setClients(clients().map(c => c.id === data.payload[0] ? { ...c, alias: data.payload[1] } : c));
      });
      break;
    
    case "MediaCreated":
      batch(() => {
        log(`Media ${data.payload.name} has been uploaded`);

        setMedia([...media(), { ...data.payload, downloaded: [] }]);
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
    
    case "MediaStarted":
      batch(() => {
        log(`Media ${data.payload.media} has been started by client ${data.payload.client}`);
        setPlaying([...playing(), data.payload.client]);
      });
      break;
    
    case "MediaStopped":
      batch(() => {
        log(`Media ${data.payload} has been stopped`);
        setPlaying(playing().filter(p => p !== data.payload));
      });
      break;

    case "GroupCreated":
      batch(() => {
        log(`Group has been created`);
        setGroups([{ id: data.payload, members: [], name: "Group #" + data.payload, color: 16106215 }, ...groups()]); // Color: #f5c2e7
      });
      break;

    default:
      console.log("Unhandled event", data);
  }
}

let callback: (state: boolean) => void;
let connected: boolean = false;
function connect() {
  sse = new EventSource(`${base_url}/api/dashboard/stream`);
  sse.onopen = () => {
    log("Connected to the server");
    connected = true;
  };
  sse.onmessage = (e) => handle(e.data);
  sse.onerror = () => {
    if (connected) log("Disconnected from server");
    connected = false;
    sse?.close();
    callback?.(false);
    setTimeout(connect, 1500);
  };
}

export function useStream() {
  const [ ready, setReady ] = createSignal(false);
  
  connect();
  callback = is_connected => setReady(is_connected);
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

export async function request_download_selected(id: number, clients: number[]) {
  log(`Media ${id} has been requested for download for clients ${clients}`);
  await fetch(`${base_url}/api/media/${id}/request_download`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(clients),
  });
}

export async function start_media(id: number, clients: number[]) {
  log(`Media ${id} has been requested to start for clients ${clients}`);
  await fetch(`${base_url}/api/media/${id}/play`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(clients),
  });
}

export async function stop_media(clients: number[]) {
  log(`Media has been requested to stop for clients ${clients}`);
  await fetch(`${base_url}/api/media/stop`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(clients),
  });
}

export async function delete_client(id: number) {
  log(`Client ${id} has been set for deletion`);
  await fetch(`${base_url}/api/client/${id}`, { method: "DELETE" });
}

export async function shutdown_client(id: number) {
  log(`Client ${id} has been requested to shutdown`);
  await fetch(`${base_url}/api/client/${id}/shutdown`, { method: "POST" });
}

export async function add_group() {
  log(`Group has been set for creation`);
  await fetch(`${base_url}/api/group`, { method: "POST" });
}

export async function change_alias(id: number, alias: string) {
  log(`Client ${id} alias has been changed to ${alias}`);
  
  const nonce = get_nonce();
  change_alias_nonce.push(nonce);
  setTimeout(() => change_alias_nonce.shift(), 1000 * 60);

  await fetch(`${base_url}/api/client/${id}/rename`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      alias,
      nonce,
    }),
  });
}
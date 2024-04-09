import { Accessor, For, Setter, Show, createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { change_alias, delete_client, onActivityChange, request_download_selected, shutdown_client, start_media, stop_media, useClients, useMedia, usePlaying } from '../stream';
import { AiOutlineCloudSync, AiTwotoneDelete } from "solid-icons/ai";
import { FaSolidPause } from "solid-icons/fa";
import { FiPlay, FiPower } from "solid-icons/fi";
import { IoMusicalNotesOutline } from "solid-icons/io";
import { VsChromeClose } from "solid-icons/vs";
import { Portal } from "solid-js/web";
import styles from "./home.module.scss";

export default function Home() {
  const [ selected, setSelected ] = createSignal<number[]>([]);
  const [ modal, setModal ] = createSignal<boolean>(false);
  const [ clients ] = useClients();
  const [ playing ] = usePlaying();
  const [ media ] = useMedia();

  const destroy = onActivityChange((activity, id) => {
    if (activity === "Offline" && selected().includes(id)) {
      setSelected(selected => selected.filter(client_id => client_id !== id));
    }
  });

  const clients_sorted = () => {
    const online = clients().filter(client => client.activity.activity === "Online");
    const offline = clients().filter(client => client.activity.activity === "Offline");

    return [ ...online, ...offline ];
  }

  createEffect(() => {
    if (selected().length === 0 && modal()) {
      setModal(false);
    }
  });

  onMount(() => {
    document.addEventListener("keydown", onKeyDown);
  });

  onCleanup(() => {
    document.removeEventListener("keydown", onKeyDown);
    destroy();
  });

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      setSelected([]);
    }
  }

  async function play(id: number) {
    await start_media(id, selected());
    setSelected([]);
  }

  async function stop() {
    await stop_media(selected());
    setSelected([]);
  }

  const is_playing_selected = () => selected().some(id => playing().includes(id));
  const is_selected_not_playing = () => selected().some(id => !playing().includes(id));

  return (
    <div class={styles.home} onClick={() => setSelected([])}>
      <Portal>
        <div class={styles.modalWrapper} onClick={() => modal() && setModal(false)} classList={{ [styles.hidden]: !modal() }}>        
          <div class={styles.modal} classList={{ [styles.hidden]: !modal() }} onClick={event => event.stopPropagation()}>
            <div class={styles.modalHeader}>
              <h1>Select a media from the library</h1>
              <div class={styles.close} onClick={() => setModal(false)}>
                <VsChromeClose />
              </div>
            </div>
            <div class={styles.list}>
              <For each={media()}>
                {item => (
                  <ModalEntry
                  id={item.id}
                  name={item.name}
                  length={item.length}
                  downloaded={item.downloaded}
                  selected={selected}
                  play={play}
                  />
                )}
              </For>
            </div>
          </div>
        </div>
      </Portal>
      <Show when={clients().length > 0} fallback={<Fallback />}>
        <div class={styles.grid}>
          <div class={styles.header} onClick={e => e.stopPropagation()} />
          <div class={styles.header} onClick={e => e.stopPropagation()}>
            Alias
          </div>
          <div class={styles.header} onClick={e => e.stopPropagation()}>
            Hostname
          </div>
          <div class={styles.header} onClick={e => e.stopPropagation()}>
            Username
          </div>
          <div class={styles.header} onClick={e => e.stopPropagation()}>
            IP
          </div>
          <div class={styles.header} onClick={e => e.stopPropagation()}>
            Activity
          </div>
          <div class={`${styles.iconWrapper} ${styles.header}`} onClick={e => e.stopPropagation()}>
            <div
              class={`${styles.icon} ${styles.play}`}
              classList={{ [styles.disabled]: !is_playing_selected() }}
              onClick={() => is_playing_selected() && stop()}
            >
              <FaSolidPause />
            </div>
          </div>
          <div class={`${styles.iconWrapper} ${styles.header}`} onClick={e => e.stopPropagation()}>
            <div
              class={`${styles.icon} ${styles.play} ${styles.disabled}`}
              classList={{ [styles.disabled]: !is_selected_not_playing() }}
              onClick={() => is_selected_not_playing() && setModal(true)}
            >
              <FiPlay />
            </div>
          </div>
          <For each={clients_sorted()}>
            {client => (
              <Client
                id={client.id}
                hostname={client.hostname}
                username={client.username}
                ip={client.ip}
                activity={client.activity.activity}
                alias={client.alias}
                selected={selected}
                setSelected={setSelected}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

function Fallback() {
  return (
    <div class={styles.fallback}>
      <h1>(╯°□°)╯︵ ┻━┻</h1>
      <h2> No clients available. </h2>
      <h3> Install the client on the device of your choice. </h3>
    </div>
  );
}

type ModalEntryProps = {
  id: number;
  name: string;
  length: number;
  downloaded: number[];
  selected: Accessor<number[]>;
  play: (id: number) => void;
}

function human_time(ms: number) {
  const min = Math.floor(ms / 60000);
  const sec = (ms % 60000) / 1000;

  return min > 0 ? `${min}m ${Math.floor(sec)}s` : `${+sec.toFixed(2)}s`;
}

function ModalEntry(props: ModalEntryProps) {
  const [ cooldown, setCooldown ] = createSignal(false);
  
  const downloaded = () => props.selected().filter(client => props.downloaded.includes(client)).length;
  const is_available = () => downloaded() === props.selected().length;

  let cooldown_timeout: number | null = null;

  async function sync() {
    if (cooldown()) return;

    setCooldown(true);
    cooldown_timeout = setTimeout(() => setCooldown(false), 1000 * 15);
    const to_download = props.selected().filter(client => !props.downloaded.includes(client));
    await request_download_selected(props.id, to_download);
  }

  onCleanup(() => {
    if (cooldown_timeout !== null) clearTimeout(cooldown_timeout);
  });

  return (
    <div class={styles.item} onClick={() => is_available() && props.play(props.id)} classList={{ [styles.available]: is_available() }}>
      <div> {props.name} </div>
      <div> {downloaded()} / {props.selected().length} </div>
      <div> {human_time(props.length)} </div>
      <div class={styles.resyncIconWrapper}>
        <div class={styles.resync} classList={{ [styles.hidden]: is_available(), [styles.cooldown]: cooldown() }} onClick={sync}>
          <AiOutlineCloudSync />
        </div>
      </div>
    </div>
  );
}

type ClientProps = {
  id: number;
  hostname: string;
  username: string;
  ip: string;
  activity: string;
  alias: string | null;
  selected: Accessor<number[]>;
  setSelected: Setter<number[]>;
};

function Client(props: ClientProps) {
  const [ dpending, setDPending ] = createSignal(false); // Delete pending
  const [ spending, setSPending ] = createSignal(false); // Shutdown pending
  const [ alias, setAlias ] = createSignal<string>(props.alias ?? "");
  const [ playing ] = usePlaying();

  let timeout: number | null = null;
  function on_input(event: Event) {
    setAlias((event.target as HTMLInputElement).value);
    if (timeout !== null) clearTimeout(timeout);

    timeout = setTimeout(() => {
      change_alias(props.id, alias());
      timeout = null;
    }, 500);
  }

  onCleanup(() => {
    if (timeout !== null) clearTimeout(timeout);
  });

  function select(event: Event) {
    event.stopPropagation();
    if (props.activity === "Offline") return;
    props.setSelected(selected => selected.includes(props.id) ? selected.filter(id => id !== props.id) : [ ...selected, props.id ]);
  }

  const is_selected = () => props.selected().includes(props.id);

  function remove(event: Event) {
    event.stopPropagation();
    if (dpending()) return;

    setDPending(true);
    delete_client(props.id);
  }

  function shutdown(event: Event) {
    event.stopPropagation();
    if (spending() || dpending() || props.activity === "Offline") return;

    setSPending(true);
    shutdown_client(props.id);
  }

  return (
    <>
      <div class={styles.iconPlaying} onClick={select} classList={{ [styles.selected]: is_selected(), [styles.hidden]: !playing().includes(props.id) }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        <IoMusicalNotesOutline />
      </div>
      <div onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        <input class={styles.input} type="text" value={alias()} onInput={on_input} placeholder={`N/A (${props.hostname})`} /> 
      </div>
      <div onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        {props.hostname}
      </div>
      <div onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        {props.username}
      </div>
      <div onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        {props.ip}
      </div>
      <div onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        <div class={styles.badge} classList={{ [styles.online]: props.activity === "Online", [styles.offline]: props.activity === "Offline" }}>
          {props.activity}
        </div>
      </div>
      <div class={styles.iconWrapper} onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        <div
          class={`${styles.icon} ${styles.delete}`}
          classList={{ [styles.disabled]: dpending() || spending() || props.activity === "Offline" }}
          onClick={shutdown}
        >
          <FiPower />
        </div>
      </div>

      <div class={styles.iconWrapper} onClick={select} classList={{ [styles.selected]: is_selected() }} style={{ cursor: props.activity === "Offline" ? "default" : "pointer" }}>
        <div
          class={`${styles.icon} ${styles.delete}`}
          classList={{ [styles.disabled]: dpending() }}
          onClick={remove}
        >
          <AiTwotoneDelete />
        </div>
      </div> 
    </>
  );
}

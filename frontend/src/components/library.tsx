import { Accessor, For, Show, createSignal, onCleanup } from "solid-js";
import { useMedia, useClients, useTempMedia, base_url, delete_media, request_download } from '../stream';
import { FaSolidPause } from "solid-icons/fa";
import { FiPlay } from "solid-icons/fi";
import { AiOutlineCloudSync, AiTwotoneDelete } from "solid-icons/ai";
import styles from "./library.module.scss";

let audio: HTMLAudioElement | null = null;

export default function Library() {
  const [ playing, setPlaying ] = createSignal<number>(-1);
  
  const [ media ] = useMedia();
  const [ tempMedia ] = useTempMedia();
  
  onCleanup(() => {
    if (audio !== null) {
      audio.onended = null;
      audio.pause();
      audio.remove();
      audio = null;
    }
  });

  return (
    <div class={styles.library}>
      <Show when={media().length > 0 || tempMedia().length > 0} fallback={<Fallback />}>
        <div class={styles.grid}>
          <div class={styles.header}>
            Name
          </div>
          <div class={styles.header} />
          <div class={styles.header} />
          <div class={styles.header} />
          <div class={styles.header} />
          <div class={styles.header} />
          <For each={media()}>
            {item => (
              <Entry
                id={item.id}
                name={item.name}
                downloaded={item.downloaded}
                length={item.length}
                playing={playing}
                setPlaying={setPlaying}
              />
            )}
          </For>
          <For each={tempMedia()}>
            {item => (
              <TempEntry name={item.name} />
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
      <h2> No media available. </h2>
      <h3> Upload some media to get started. </h3>
    </div>
  );
}

type TempEntryProps = {
  name: string;
};

function TempEntry({ name }: TempEntryProps) {
  return (
    <>
      <div> {name} </div>
      <div> ??? </div>
      <div> ??? </div>
      <div class={styles.iconWrapper}> <div class={`${styles.icon} ${styles.play} ${styles.disabled}`}> <FiPlay /> </div> </div>
      <div class={styles.iconWrapper}> <div class={`${styles.icon} ${styles.sync} ${styles.disabled}`}> <AiOutlineCloudSync /> </div> </div>
      <div class={styles.iconWrapper}> <div class={`${styles.icon} ${styles.delete} ${styles.disabled}`}> <AiTwotoneDelete /> </div> </div>
    </>
  );
}

function human_time(ms: number) {
  const min = Math.floor(ms / 60000);
  const sec = (ms % 60000) / 1000;

  return min > 0 ? `${min}m ${Math.floor(sec)}s` : `${+sec.toFixed(2)}s`;
}

type EntryProps = {
  id: number;
  name: string;
  downloaded: number[];
  length: number;
  playing: Accessor<number>;
  setPlaying: (id: number) => void;
};

function Entry({ id, name, downloaded, length, playing, setPlaying }: EntryProps) {
  const [ pending, setPending ] = createSignal(false);
  const [ refreshCd, setRefreshCd ] = createSignal(false);
  const [ clients ] = useClients();

  let refreshTimeout: number | null = null;

  onCleanup(() => {
    if (refreshTimeout !== null) clearTimeout(refreshTimeout);
  });

  async function refresh() {
    if (pending() || refreshCd()) return;

    setRefreshCd(true);
    refreshTimeout = setTimeout(() => setRefreshCd(false), 1000 * 30);
    await request_download(id);
  }

  async function remove() {
    if (pending()) return;
    
    if (playing() === id) {
      setPlaying(-1);
      audio!.onended = null;
      audio!.pause();
      audio?.remove();
      audio = null;
    }

    setPending(true);
    await delete_media(id);
  }

  function play() {
    if (pending()) return;

    if (audio !== null) {
      audio.onended = null;
      audio.pause();
      audio.remove();
      audio = null;
    
      if (playing() === id) {
        setPlaying(-1);
        return;
      };
    }

    setPlaying(id);
    audio = new Audio(`${base_url}/api/media/${id}`);
    audio.play();
    audio.onended = () => {
      setPlaying(-1);
      audio?.remove();
      audio = null;
    };
  }

  return (
    <>
      <div> {name} </div>
      <div> {downloaded.length} / {clients().length} </div>
      <div> {human_time(length)} </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.play}`}
          classList={{ [styles.disabled]: pending() }}
          onClick={play}
        >
        { playing() === id
          ? <FaSolidPause />
          : <FiPlay />
        }
        </div>
      </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.sync}`}
          classList={{ [styles.disabled]: pending() || refreshCd() }}
          onClick={refresh}
        >
          <AiOutlineCloudSync />
        </div>
      </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.delete}`}
          classList={{ [styles.disabled]: pending() }}
          onClick={remove}
        >
          <AiTwotoneDelete />
        </div>
      </div> 
    </>
  );
}

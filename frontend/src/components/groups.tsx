import { For } from "solid-js";
import { useClients, useGroups, usePlaying } from "../stream";
import { AiTwotoneDelete } from "solid-icons/ai";
import { FaSolidPause } from "solid-icons/fa";
import { FiPlay, FiPower } from "solid-icons/fi";
import styles from "./groups.module.scss";

export default function Groups() {
  const [ groups ] = useGroups();

  return (
    <div class={styles.groups}>
      <div class={styles.grid}>
        <div class={styles.header}></div>
        <div class={styles.header}>
          Name
        </div>
        <div class={styles.header}>
          Activity
        </div>
        <div class={styles.header}>
          Playing
        </div>
        <div class={styles.header}></div>
        <div class={styles.header}></div>
        <div class={styles.header}></div>
        <div class={styles.header}></div>
        <For each={groups()}>
          {item => (
            <Entry
              id={item.id}
              name={item.name}
              members={item.members}
              color={item.color}
            />
          )}
        </For>
      </div>
    </div>
  );
}

function colorize(color: number) {
  return `rgb(${color >> 16}, ${(color >> 8) & 0xff}, ${color & 0xff})`;
}

type EntryProps = {
  id: number;
  name: string;
  members: number[];
  color: number;
}

export function Entry({ name, members, color }: EntryProps) {
  const [ clients ] = useClients();
  const [ playing ] = usePlaying();

  const get_active = () => clients().filter(client => members.includes(client.id) && client.activity.activity === "Online");
  const get_playing = () => playing().filter(client_id => members.includes(client_id));

  return (
    <>
      <div>
        <div class={styles.color} style={{ "background-color": colorize(color) }} />
      </div>
      <div>
        <input type="text" name="" id="" value={name} />
      </div>
      <div> {get_active().length} / {members.length} </div>
      <div> {get_playing().length} / {members.length} </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.play}`}
        >
          <FiPlay />
        </div>
      </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.play}`}
        >
          <FaSolidPause />
        </div>
      </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.delete}`}
        >
          <FiPower />
        </div>
      </div>
      <div class={styles.iconWrapper}>
        <div
          class={`${styles.icon} ${styles.delete}`}
        >
          <AiTwotoneDelete />
        </div>
      </div>
    </>
  )
}

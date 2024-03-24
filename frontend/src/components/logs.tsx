import { For } from "solid-js";
import { useLogs, T0 } from "../stream";
import styles from "./logs.module.scss";

function human_time(ms: number) {
  const h = Math.floor(ms / 3600000)
  const m = ms / 60000 % 60;
  const s = ms / 1000 % 60;

  if (h != 0) return `${h}h ${Math.floor(m)}m ${Math.floor(s)}s`;
  if (m >= 1) return `${Math.floor(m)}m ${Math.floor(s)}s`;
  return `${s >= 10 ? +s.toFixed(2) : s}s`;
}

export default function Logs() {
  const [ logs, _ ] = useLogs();

  return (
    <div class={styles.logs}>
      <div class={styles.grid}>
        <div class={styles.header}>
          Time
        </div>
        <div class={styles.header}>
          T+
        </div>
        <div class={styles.header}>
          Message
        </div>
        <For each={logs()}>
          {log => (
            <>
              <div class={styles.time}>
                {new Date(log.time).toLocaleTimeString()}
              </div>
              <div class={styles.tplus}>
                {human_time(log.time - T0)}
              </div>
              <div class={styles.message}>
                {log.message}
              </div>
            </>
          )}
        </For>
      </div>
    </div>
  );
}
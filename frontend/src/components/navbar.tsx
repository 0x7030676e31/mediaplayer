import { useLocation, useNavigate } from "@solidjs/router";
import { AiOutlineFileAdd } from 'solid-icons/ai'
import { useClients, useMedia, add_media } from "../stream";
import styles from "./navbar.module.scss";

export default function Navbar() {
  let input: HTMLInputElement | undefined;
  
  const [ media ] = useMedia();
  const [ clients ] = useClients();
  const location = useLocation();
  const navigate = useNavigate();

  function open_dialog() {
    input?.click();
  }

  function upload() {
    const file = input?.files?.[0];
    if (!file) return;

    add_media(file);
  }

  return (
    <div class={styles.navbar}>
      <input type="file" accept="audio/*" class={styles.input} ref={input} onChange={upload} />
      <div class={styles.tabs}>
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/" }} onClick={() => navigate("/")}>
          Clients
        <div class={styles.badge}>{clients().length}</div>
        </div>
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/library" }} onClick={() => navigate("/library")}>
          Library
          <div class={styles.badge}>{media().length}</div>
        </div>
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/logs" }} onClick={() => navigate("/logs")}>
          Logs
        </div>
      </div>
      <div class={styles.icon} classList={{ [styles.show]: location.pathname === "/library" }} onClick={open_dialog}>
        <AiOutlineFileAdd />
      </div>
    </div>
  );
}

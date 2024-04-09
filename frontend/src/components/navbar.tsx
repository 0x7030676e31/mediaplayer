import { useLocation, useNavigate } from "@solidjs/router";
import { useClients, useMedia, useGroups, add_media, add_group } from "../stream";
import { AiOutlineFileAdd, AiOutlineFolderAdd } from 'solid-icons/ai'
import { Match, Switch, createSignal } from "solid-js";
import styles from "./navbar.module.scss";

export default function Navbar() {
  const [ disabled, setDisabled ] = createSignal(false);
  let input: HTMLInputElement | undefined;

  const [ media ] = useMedia();
  const [ clients ] = useClients();
  const [ _groups ] = useGroups();
  const location = useLocation();
  const navigate = useNavigate();

  async function icon_action() {
    if (location.pathname === "/mp/library") {
      input?.click();
      return;
    }

    if (location.pathname === "/mp/groups" && !disabled()) {
      setDisabled(true);
      await add_group();
      setDisabled(false);
    }
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
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/mp" }} onClick={() => navigate("/mp")}>
          Clients
        <div class={styles.badge}>{clients().length}</div>
        </div>
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/mp/library" }} onClick={() => navigate("/mp/library")}>
          Library
          <div class={styles.badge}>{media().length}</div>
        </div>
        {/* <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/mp/groups" }} onClick={() => navigate("/mp/groups")}>
          Groups
        <div class={styles.badge}>{groups().length}</div>
        </div> */}
        <div class={styles.tab} classList={{ [styles.active]: location.pathname === "/mp/logs" }} onClick={() => navigate("/mp/logs")}>
          Logs
        </div>
      </div>
      <div class={styles.icon} onClick={icon_action}>
        <Switch>
          <Match when={location.pathname === "/mp/library"}>
            <AiOutlineFileAdd />
          </Match>
          <Match when={location.pathname === "/mp/groups"}>
            <AiOutlineFolderAdd classList={{ [styles.disabled]: disabled() }} />
          </Match>
        </Switch>
      </div>

      {/* <div class={styles.icon} classList={{ [styles.show]: location.pathname === "/mp/library" }} onClick={open_dialog}>
        <AiOutlineFileAdd /> <AiOutlineFolderAdd />
      </div> */}
      {/* swtich */}
    </div>
  );
}

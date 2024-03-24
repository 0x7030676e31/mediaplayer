import styles from "./home.module.scss";
import { useClients } from '../stream';

export default function Home() {
  const [ clients, _ ] = useClients();

  const clients_sorted = () => {
    const online = clients().filter(client => client.activity.activity === "Online");
    const offline = clients().filter(client => client.activity.activity === "Offline");

    return [ ...online, ...offline ];
  }
  <div></div>

  return (
    <div class={styles.home}>
      <div class={styles.grid}>
        <div class={styles.header}>
          {/* Checkbox */}
        </div>
        <div class={styles.header}>
          Hostname
        </div>
        <div class={styles.header}>
          Username
        </div>
        <div class={styles.header}>
          IP
        </div>
        <div class={styles.header}>
          Activity
        </div>
        <div></div>
        <div></div>
      </div>
    </div>
  );
}

function fallback() {
  return (
    <></>
  );
}

type ClientProps = {

};

function Client(props: ClientProps) {
  return (
    <div></div>
  );
}

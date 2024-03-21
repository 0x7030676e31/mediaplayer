import { RouteSectionProps } from "@solidjs/router";
import { Accessor } from "solid-js";
import { useStream } from "./stream";
import Navbar from './components/navbar';

export default function App(props: RouteSectionProps) {
  const { connected } = useStream();

  return (
    <div class="app">
      <Navbar />
      {props.children}
      <Overlay connected={connected} />
    </div>
  );
}

function Overlay({ connected }: { connected: Accessor<boolean> }) {
  return (
    <div class="overlay" classList={{ "connected": connected() }}>
      <svg class="spinner" viewBox="0 0 50 50">
        <circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="5" />
      </svg>
    </div>
  );
}


  // const { connected } = useStream();

  // let input: HTMLInputElement | undefined;

  // Home
  // Library
  
  // function onClick() {
  //   input?.click();
  // }

  // async function upload(e: Event) {
  //   const file = (e.target as HTMLInputElement).files?.[0];
  //   if (!file) return;

  //   const name = encodeURIComponent(file.name);
  //   await fetch(`http://localhost:7777/api/media/${name}`, {
  //     method: "POST",
  //     body: file,
  //   });
  // }

  // async function connect() {
  //   const sse = new EventSource("http://localhost:7777/api/dashboard/stream");
  //   sse.onmessage = (e) => {
  //     console.log("Unhlandled message:", e.data);
  //   };
  // }

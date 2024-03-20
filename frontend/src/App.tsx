export default function App() {
  let input: HTMLInputElement | undefined;

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

  return (
    <>
      <h1>Upload a file</h1>
      {/* <button onClick={onClick}>Choose file</button>
      <input type="file" style={{ display: "none" }} ref={input} onChange={upload} accept="audio/*" />
      <button onClick={connect}>Connect to server</button> */}
    </>
  );
}

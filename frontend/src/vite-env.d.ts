/// <reference types="vite/client" />

declare global {
  interface Media {
    id: number;
    name: string;
    downloaded: number[];
  }

  interface Client {
    id: number;
    ip: string;
    hostname: string;
    activity: Activity;
  }

  type Activity = { activity: Online } | { activity: Offline, timestamp: number };
}

export {}

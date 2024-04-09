/// <reference types="vite/client" />

import { Accessor } from 'solid-js';

declare global {
  interface Window {
    get_media: Accessor<Media[]>;
    get_clients: Accessor<Client[]>;
    get_groups: Accessor<Group[]>;
  }
  
  interface Media {
    id: number;
    name: string;
    downloaded: number[];
    length: number;
  }

  interface Client {
    id: number;
    ip: string;
    hostname: string;
    username: string;
    activity: Activity;
    alias: string | null;
  }

  interface Group {
    id: number;
    name: string;
    members: number[];
    color: number;
  }

  type Activity = { activity: "Online" } | { activity: "Offline", timestamp: number };
}

export {}

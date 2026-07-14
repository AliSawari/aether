import { writable } from "svelte/store";
import type { ServerInfo, WgStatus, NetworkInfo } from "./tauri";

export type Page = "home" | "configs" | "about";

export const page = writable<Page>("home");
export const servers = writable<ServerInfo[]>([]);
export const status = writable<WgStatus>({
  connected: false,
  interface: null,
  server_index: null,
  endpoint: null,
  latest_handshake: null,
  transfer: { rx: "0 B", tx: "0 B" },
});
export const network = writable<NetworkInfo | null>(null);
export const workspace = writable<string | null>(null);
export const selectedServer = writable<number | null>(null);
export const passwordless = writable(false);
export const loading = writable(false);
export const toasts = writable<{ id: number; message: string; type: "success" | "error" }[]>([]);

let toastId = 0;

export function showToast(message: string, type: "success" | "error" = "success") {
  const id = ++toastId;
  toasts.update((t) => [...t, { id, message, type }]);
  setTimeout(() => {
    toasts.update((t) => t.filter((x) => x.id !== id));
  }, 4000);
}

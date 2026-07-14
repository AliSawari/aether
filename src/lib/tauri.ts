import { invoke } from "@tauri-apps/api/core";

export interface ServerInfo {
  index: number;
  file: string;
  host: string;
  port: string;
  address: string;
}

export interface TransferInfo {
  rx: string;
  tx: string;
}

export interface WgStatus {
  connected: boolean;
  interface: string | null;
  server_index: number | null;
  endpoint: string | null;
  latest_handshake: string | null;
  transfer: TransferInfo;
}

export interface NetworkInfo {
  gateway: string;
  interface_wifi: string;
  interface_eth: string;
}

export const api = {
  getWorkspace: () => invoke<string | null>("get_workspace"),
  setWorkspace: (path: string) => invoke<ServerInfo[]>("set_workspace_cmd", { path }),
  getServers: () => invoke<ServerInfo[]>("get_servers"),
  getStatus: () => invoke<WgStatus>("get_status"),
  getNetworkInfo: () => invoke<NetworkInfo>("get_network_info"),
  getSelectedServer: () => invoke<number | null>("get_selected_server"),
  setSelectedServer: (index: number) => invoke<void>("set_selected_server", { index }),
  elevationStatus: () => invoke<boolean>("elevation_status"),
  installElevation: () => invoke<void>("install_elevation"),
  connect: (server: number) => invoke<string>("connect", { server }),
  disconnect: () => invoke<string>("disconnect"),
  restart: (server: number) => invoke<string>("restart", { server }),
  regenerateConfigs: () => invoke<void>("regenerate_configs"),
  importConfig: (path: string) => invoke<ServerInfo[]>("import_config", { path }),
  openWorkspaceFolder: () => invoke<void>("open_workspace_folder"),
};

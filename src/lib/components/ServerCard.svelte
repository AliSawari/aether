<script lang="ts">
  import { Power, RefreshCw, Shield } from "lucide-svelte";
  import type { ServerInfo, WgStatus } from "../tauri";
  import { api } from "../tauri";
  import { loading, selectedServer, showToast } from "../stores";

  let {
    server,
    status,
    onUpdate,
  }: {
    server: ServerInfo;
    status: WgStatus;
    onUpdate: () => void | Promise<void>;
  } = $props();

  const isActive = $derived(status.connected && status.server_index === server.index);

  async function run(action: () => Promise<unknown>, success: string, opts?: { select?: boolean }) {
    loading.set(true);
    try {
      await action();
      if (opts?.select) {
        selectedServer.set(server.index);
        try {
          await api.setSelectedServer(server.index);
        } catch {
          /* ignore */
        }
      }
      showToast(success);
      await onUpdate();
    } catch (e) {
      showToast(String(e), "error");
      await onUpdate();
    } finally {
      loading.set(false);
    }
  }
</script>

<div class="card" class:active={isActive}>
  <div class="top">
    <Shield size={18} color={isActive ? "var(--accent)" : "var(--muted)"} />
    <div>
      <div class="host">{server.host}</div>
      <div class="mono muted meta">:{server.port} · {server.address}</div>
    </div>
    <span class="idx mono muted">#{server.index}</span>
  </div>

  {#if isActive}
    <div class="mono muted transfer">
      ↓ {status.transfer.rx} · ↑ {status.transfer.tx}
      {#if status.latest_handshake}
        · {status.latest_handshake}
      {/if}
    </div>
  {/if}

  <div class="server-actions">
    {#if isActive}
      <button
        class="btn btn-danger"
        disabled={$loading}
        onclick={() => run(() => api.disconnect(), "Disconnected")}
      >
        <Power size={14} /> Disconnect
      </button>
      <button
        class="btn btn-ghost"
        disabled={$loading}
        onclick={() => run(() => api.restart(server.index), "Restarted")}
      >
        <RefreshCw size={14} /> Restart
      </button>
    {:else}
      <button
        class="btn btn-primary"
        disabled={$loading || (status.connected && !isActive)}
        onclick={() =>
          run(() => api.connect(server.index), `Connected to ${server.host}`, { select: true })
        }
      >
        <Power size={14} /> Connect
      </button>
    {/if}
  </div>
</div>

<style>
  .top {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }
  .host {
    font-size: 1.05rem;
    font-weight: 600;
  }
  .meta {
    font-size: 0.8rem;
    margin-top: 0.15rem;
  }
  .idx {
    margin-left: auto;
    font-size: 0.85rem;
  }
  .transfer {
    font-size: 0.75rem;
    margin-top: 0.6rem;
  }
</style>

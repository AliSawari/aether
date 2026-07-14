<script lang="ts">
  import { Power } from "lucide-svelte";
  import {
    servers,
    status,
    selectedServer,
    loading,
    page,
    showToast,
  } from "../stores";
  import { api } from "../tauri";

  const selected = $derived(
    $servers.find((s) => s.index === $selectedServer) ?? $servers[0] ?? null,
  );

  async function onSelect(e: Event) {
    const value = Number((e.target as HTMLSelectElement).value);
    selectedServer.set(value);
    try {
      await api.setSelectedServer(value);
    } catch {
      /* ignore */
    }
  }

  async function toggle() {
    if (!selected) return;
    loading.set(true);
    try {
      if ($status.connected && $status.server_index === selected.index) {
        await api.disconnect();
        showToast("Disconnected");
      } else {
        if ($status.connected) {
          await api.disconnect();
        }
        await api.connect(selected.index);
        showToast(`Connected to ${selected.host}`);
      }
    } catch (e) {
      showToast(String(e), "error");
    } finally {
      loading.set(false);
    }
  }

  const isOn = $derived(
    $status.connected && selected != null && $status.server_index === selected.index,
  );
</script>

<section class="home">
  {#if $servers.length === 0}
    <div class="empty card">
      <p>No configs loaded yet.</p>
      <p class="muted">Import provider <code>.conf</code> files in Configurations first.</p>
      <button class="btn btn-primary" onclick={() => page.set("configs")}>
        Go to Configurations
      </button>
    </div>
  {:else}
    <label class="select-wrap muted" for="cfg-select">Server</label>
    <select id="cfg-select" class="select" value={selected?.index ?? ""} onchange={onSelect}>
      {#each $servers as s (s.index)}
        <option value={s.index}>{s.host} — {s.file}</option>
      {/each}
    </select>

    <button
      class="power"
      class:on={isOn}
      disabled={$loading || !selected}
      onclick={toggle}
      aria-label={isOn ? "Power off" : "Power on"}
    >
      <Power size={64} strokeWidth={1.75} />
    </button>

    <div class="state" class:on={isOn}>
      {isOn ? "CONNECTED" : "DISCONNECTED"}
    </div>

    {#if selected}
      <div class="mono muted detail">{selected.host}:{selected.port}</div>
    {/if}

    {#if isOn}
      <div class="mono muted transfer">
        ↓ {$status.transfer.rx} · ↑ {$status.transfer.tx}
      </div>
    {/if}
  {/if}
</section>

<style>
  .home {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 1rem;
  }
  .empty {
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    align-items: center;
    max-width: 280px;
  }
  .select-wrap {
    align-self: stretch;
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .select {
    align-self: stretch;
    appearance: none;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    border-radius: var(--radius-btn);
    padding: 0.7rem 0.9rem;
    font-family: "JetBrains Mono", monospace;
    font-size: 0.85rem;
  }
  .select:focus {
    outline: none;
    border-color: var(--accent);
  }
  .power {
    width: 148px;
    height: 148px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    margin: 1rem 0 0.25rem;
    background: radial-gradient(circle at 40% 35%, #1c2230 0%, #12151c 70%);
    border: 2px solid var(--border);
    color: var(--muted);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.45);
    transition: all 0.25s ease;
  }
  .power:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
    transform: scale(1.03);
  }
  .power.on {
    border-color: var(--accent);
    color: var(--accent);
    box-shadow: 0 0 40px var(--accent-dim), 0 8px 32px rgba(0, 0, 0, 0.45);
  }
  .power:disabled {
    opacity: 0.45;
  }
  .state {
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: 0.2em;
    color: var(--muted);
  }
  .state.on {
    color: var(--accent);
  }
  .detail,
  .transfer {
    font-size: 0.8rem;
  }
  code {
    font-family: "JetBrains Mono", monospace;
    color: var(--accent);
  }
</style>

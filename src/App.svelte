<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import StatusPill from "./lib/components/StatusPill.svelte";
  import WorkspaceBar from "./lib/components/WorkspaceBar.svelte";
  import NetworkBar from "./lib/components/NetworkBar.svelte";
  import WorkspacePicker from "./lib/components/WorkspacePicker.svelte";
  import ServerCard from "./lib/components/ServerCard.svelte";
  import ActionBar from "./lib/components/ActionBar.svelte";
  import HomePage from "./lib/components/HomePage.svelte";
  import AboutPage from "./lib/components/AboutPage.svelte";
  import Toast from "./lib/components/Toast.svelte";
  import { api } from "./lib/tauri";
  import {
    servers,
    status,
    network,
    workspace,
    page,
    selectedServer,
    passwordless,
  } from "./lib/stores";

  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function refresh() {
    try {
      const [s, st, net, elev] = await Promise.all([
        api.getServers(),
        api.getStatus(),
        api.getNetworkInfo(),
        api.elevationStatus(),
      ]);
      servers.set(s);
      status.set(st);
      network.set(net);
      passwordless.set(elev);

      if (s.length > 0) {
        const saved = await api.getSelectedServer();
        const valid = saved && s.some((x) => x.index === saved);
        selectedServer.set(valid ? saved : s[0].index);
      }
    } catch {
      /* ignore poll errors */
    }
  }

  async function init() {
    const ws = await api.getWorkspace();
    workspace.set(ws);
    if (ws) await refresh();
  }

  function handleWorkspaceSet(e: CustomEvent) {
    workspace.set(e.detail.path);
    servers.set(e.detail.servers);
    refresh();
  }

  function handleServersUpdated(e: CustomEvent) {
    servers.set(e.detail);
    refresh();
  }

  onMount(() => {
    init();
    pollTimer = setInterval(refresh, 3000);
    window.addEventListener("workspace-set", handleWorkspaceSet as EventListener);
    window.addEventListener("servers-updated", handleServersUpdated as EventListener);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    window.removeEventListener("workspace-set", handleWorkspaceSet as EventListener);
    window.removeEventListener("servers-updated", handleServersUpdated as EventListener);
  });
</script>

<div class="shell">
  {#if $workspace}
    <Sidebar />
  {/if}

  <div class="main">
    <header class="header">
      {#if !$workspace}
        <span class="logo">AETHER</span>
      {:else}
        <span class="page-title">
          {$page === "home" ? "Home" : $page === "configs" ? "Configurations" : "About"}
        </span>
      {/if}
      <StatusPill connected={$status.connected} />
    </header>

    {#if !$workspace}
      <WorkspacePicker />
    {:else if $page === "home"}
      <HomePage />
    {:else if $page === "configs"}
      <WorkspaceBar />
      <NetworkBar info={$network} />
      <div class="server-list">
        {#if $servers.length === 0}
          <div class="card muted" style="text-align:center;padding:2rem">
            No provider <code>.conf</code> files in this folder. Import one or pick a folder that already has them.
          </div>
        {:else}
          {#each $servers as server (server.index)}
            <ServerCard {server} status={$status} onUpdate={refresh} />
          {/each}
        {/if}
      </div>
      <ActionBar onUpdate={refresh} />
    {:else}
      <AboutPage />
    {/if}
  </div>

  <Toast />
</div>

<style>
  code {
    font-family: "JetBrains Mono", monospace;
    color: var(--accent);
  }
  .page-title {
    font-size: 1.35rem;
    font-weight: 700;
    letter-spacing: 0.06em;
  }
</style>

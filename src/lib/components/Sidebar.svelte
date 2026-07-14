<script lang="ts">
  import { Home, Settings2, Info, KeyRound } from "lucide-svelte";
  import { page, passwordless, loading, showToast } from "../stores";
  import { api } from "../tauri";
  import type { Page } from "../stores";

  const links: { id: Page; label: string; icon: typeof Home }[] = [
    { id: "home", label: "Home", icon: Home },
    { id: "configs", label: "Configurations", icon: Settings2 },
    { id: "about", label: "About", icon: Info },
  ];

  async function authorize() {
    loading.set(true);
    try {
      await api.installElevation();
      passwordless.set(true);
      showToast("Authorized — password no longer required");
    } catch (e) {
      showToast(String(e), "error");
    } finally {
      loading.set(false);
    }
  }
</script>

<aside class="sidebar">
  <div class="brand">
    <img src="/aether.svg" alt="" width="28" height="28" />
    <span>AETHER</span>
  </div>
  <nav>
    {#each links as link}
      <button
        class="nav-item"
        class:active={$page === link.id}
        onclick={() => page.set(link.id)}
      >
        <link.icon size={18} />
        <span>{link.label}</span>
      </button>
    {/each}
  </nav>

  <div class="sidebar-foot">
    {#if !$passwordless}
      <button class="btn btn-ghost auth-btn" disabled={$loading} onclick={authorize}>
        <KeyRound size={14} />
        Authorize once
      </button>
      <p class="hint muted">Enter password once — then connect without prompts.</p>
    {:else}
      <p class="hint muted authorized">Passwordless elevation active</p>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 168px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1.25rem 0.85rem;
    border-right: 1px solid var(--border);
    background: rgba(13, 15, 20, 0.45);
    backdrop-filter: blur(8px);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    font-size: 1.15rem;
    font-weight: 700;
    letter-spacing: 0.28em;
    padding: 0 0.4rem;
  }
  .brand img {
    flex-shrink: 0;
    letter-spacing: 0;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.55rem 0.65rem;
    border-radius: var(--radius-btn);
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--muted);
    text-align: left;
    transition: all 0.15s ease;
  }
  .nav-item:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.04);
  }
  .nav-item.active {
    color: var(--accent);
    background: var(--accent-dim);
  }
  .sidebar-foot {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .auth-btn {
    width: 100%;
    justify-content: center;
    font-size: 0.8rem;
    padding: 0.4rem 0.5rem;
  }
  .hint {
    font-size: 0.7rem;
    line-height: 1.35;
    padding: 0 0.15rem;
  }
  .authorized {
    color: var(--accent);
  }
</style>

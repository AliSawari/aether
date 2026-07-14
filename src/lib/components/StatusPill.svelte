<script lang="ts">
  let { connected = false }: { connected?: boolean } = $props();
</script>

<span class="pill" class:connected>
  <span class="dot"></span>
  {connected ? "Connected" : "Disconnected"}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.75rem;
    border-radius: 999px;
    font-size: 0.85rem;
    font-weight: 600;
    border: 1px solid var(--border);
    color: var(--muted);
  }
  .pill.connected {
    border-color: var(--accent);
    color: var(--accent);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--muted);
  }
  .pill.connected .dot {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent);
    animation: pulse 2s infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>

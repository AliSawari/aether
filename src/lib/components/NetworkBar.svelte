<script lang="ts">
  import { Wifi, EthernetPort } from "lucide-svelte";
  import type { NetworkInfo } from "../tauri";

  let { info }: { info: NetworkInfo | null } = $props();
</script>

{#if info}
  <div class="card bar">
    <div class="item">
      <Wifi size={14} />
      <span class="mono muted">{info.interface_wifi}</span>
    </div>
    <div class="item">
      <EthernetPort size={14} />
      <span class="mono muted">{info.interface_eth}</span>
    </div>
    <div class="item gateway mono muted">gw {info.gateway}</div>
  </div>
{/if}

<style>
  .bar {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    padding: 0.6rem 1rem;
    font-size: 0.85rem;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    color: var(--muted);
  }
  .gateway {
    margin-left: auto;
  }
</style>

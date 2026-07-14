<script lang="ts">
  import { FolderOpen } from "lucide-svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api } from "../tauri";
  import { showToast } from "../stores";

  async function pickWorkspace() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || typeof selected !== "string") return;
    try {
      const servers = await api.setWorkspace(selected);
      showToast("Workspace ready");
      window.dispatchEvent(new CustomEvent("workspace-set", { detail: { path: selected, servers } }));
    } catch (e) {
      showToast(String(e), "error");
    }
  }
</script>

<div class="onboarding">
  <h2>Choose your WireGuard workspace</h2>
  <p class="muted">
    Pick a folder that contains your provider <code>.conf</code> files.
    Aether scans that folder only (not parents or children) and generates
    <code>smart*-wifi.conf</code> / <code>smart*-eth.conf</code> beside them.
  </p>
  <button class="btn btn-primary" onclick={pickWorkspace}>
    <FolderOpen size={18} />
    Select Folder
  </button>
</div>

<style>
  code {
    font-family: "JetBrains Mono", monospace;
    color: var(--accent);
    font-size: 0.85em;
  }
  p {
    max-width: 320px;
    line-height: 1.5;
  }
</style>

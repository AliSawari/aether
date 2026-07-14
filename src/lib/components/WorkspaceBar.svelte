<script lang="ts">
  import { FolderOpen } from "lucide-svelte";
  import { api } from "../tauri";
  import { workspace, showToast } from "../stores";
  import { open } from "@tauri-apps/plugin-dialog";

  async function changeFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || typeof selected !== "string") return;
    try {
      const servers = await api.setWorkspace(selected);
      workspace.set(selected);
      showToast("Workspace updated");
      window.dispatchEvent(new CustomEvent("servers-updated", { detail: servers }));
    } catch (e) {
      showToast(String(e), "error");
    }
  }

  function truncate(path: string) {
    if (path.length <= 36) return path;
    return "…" + path.slice(-34);
  }
</script>

<div class="card bar">
  <div class="row">
    <FolderOpen size={16} color="var(--accent)" />
    <span class="mono muted path">{$workspace ? truncate($workspace) : "No workspace"}</span>
  </div>
  <button class="btn btn-ghost" onclick={changeFolder}>Change</button>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.65rem 1rem;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>

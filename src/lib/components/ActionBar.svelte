<script lang="ts">
  import { FileUp, FolderOpen, RefreshCw } from "lucide-svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { api } from "../tauri";
  import { loading, showToast } from "../stores";

  let { onUpdate }: { onUpdate: () => void } = $props();

  async function importConfig() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "WireGuard", extensions: ["conf"] }],
    });
    if (!selected || typeof selected !== "string") return;
    loading.set(true);
    try {
      await api.importConfig(selected);
      showToast("Config imported");
      onUpdate();
    } catch (e) {
      showToast(String(e), "error");
    } finally {
      loading.set(false);
    }
  }

  async function regenerate() {
    loading.set(true);
    try {
      await api.regenerateConfigs();
      showToast("Configs regenerated");
      onUpdate();
    } catch (e) {
      showToast(String(e), "error");
    } finally {
      loading.set(false);
    }
  }

  async function openFolder() {
    try {
      await api.openWorkspaceFolder();
    } catch (e) {
      showToast(String(e), "error");
    }
  }
</script>

<div class="footer-actions">
  <button class="btn btn-ghost" disabled={$loading} onclick={importConfig}>
    <FileUp size={14} /> Import
  </button>
  <button class="btn btn-ghost" disabled={$loading} onclick={regenerate}>
    <RefreshCw size={14} /> Regenerate
  </button>
  <button class="btn btn-ghost" onclick={openFolder}>
    <FolderOpen size={14} /> Open Folder
  </button>
</div>

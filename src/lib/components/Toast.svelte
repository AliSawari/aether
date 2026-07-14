<script lang="ts">
  import { toasts } from "../stores";
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div class="toast" class:success={toast.type === "success"} class:error={toast.type === "error"}>
      {toast.message}
    </div>
  {/each}
</div>

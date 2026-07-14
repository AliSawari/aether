<script lang="ts">
  import { Github, ExternalLink } from "lucide-svelte";
</script>

<section class="about">
  <h2>About Aether</h2>
  <p class="lead">WireGuard, refined.</p>
  <p class="muted">
    Aether is a lightweight Linux desktop app for managing WireGuard connections.
    Pick a workspace, import provider configs, generate WiFi/Ethernet route-aware
    configs, and connect from a clean native UI — or the system tray.
  </p>

  <div class="card author">
    <h3>Author</h3>
    <p>Ali Sawari</p>
    <a class="link" href="https://github.com/alisawari" target="_blank" rel="noreferrer">
      <Github size={16} />
      github.com/alisawari
      <ExternalLink size={12} />
    </a>
  </div>

  <div class="card author">
    <h3>Repository</h3>
    <a class="link" href="https://github.com/alisawari/Aether" target="_blank" rel="noreferrer">
      <Github size={16} />
      github.com/alisawari/Aether
      <ExternalLink size={12} />
    </a>
  </div>
</section>

<style>
  .about {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 0.25rem 0.25rem 1rem;
  }
  h2 {
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .lead {
    font-size: 1.1rem;
    color: var(--accent);
    font-weight: 600;
  }
  p.muted {
    line-height: 1.55;
  }
  .author {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  h3 {
    font-size: 0.85rem;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .link {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    color: var(--accent);
    text-decoration: none;
    font-family: "JetBrains Mono", monospace;
    font-size: 0.82rem;
  }
  .link:hover {
    text-decoration: underline;
  }
</style>

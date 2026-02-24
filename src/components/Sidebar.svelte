<script lang="ts">
  import type { Translation } from "../types";

  let { view, text, onOpenTests, onOpenSettings } = $props<{
    view: "main" | "settings";
    text: Translation;
    onOpenTests: () => void;
    onOpenSettings: () => void;
  }>();
</script>

<aside class="sidebar">
  <div class="nav-top">
    <button
      class="nav-btn"
      data-active={view === "main"}
      onclick={onOpenTests}
      title={text.navTests}
      aria-label={text.navTests}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M7 6h10M7 12h10M7 18h10"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
  <div class="nav-bottom">
    <button
      class="nav-btn"
      data-active={view === "settings"}
      onclick={onOpenSettings}
      title={text.navSettings}
      aria-label={text.navSettings}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm7.5-3.5c0-.46-.04-.92-.12-1.35l2.02-1.57-1.9-3.3-2.4.97a7.8 7.8 0 0 0-2.34-1.35l-.37-2.55h-3.8l-.37 2.55a7.8 7.8 0 0 0-2.34 1.35l-2.4-.97-1.9 3.3 2.02 1.57c-.08.43-.12.89-.12 1.35s.04.92.12 1.35l-2.02 1.57 1.9 3.3 2.4-.97c.72.57 1.51 1.04 2.34 1.35l.37 2.55h3.8l.37-2.55c.83-.31 1.62-.78 2.34-1.35l2.4.97 1.9-3.3-2.02-1.57c.08-.43.12-.89.12-1.35Z"
          fill="currentColor"
        />
      </svg>
    </button>
  </div>
</aside>

<script lang="ts">
  import type { KeyStatePayload, Translation } from "../types";

  let {
    open,
    text,
    keyState,
  } = $props<{
    open: boolean;
    text: Translation;
    keyState: KeyStatePayload;
  }>();

  type KeyInfo = {
    label: string;
    pressed: boolean;
    icon: "back" | "down" | "up" | "confirm";
  };

  const keys = $derived<KeyInfo[]>([
    { label: text.keyTestBack, pressed: keyState.back_pressed, icon: "back" },
    { label: text.keyTestDown, pressed: keyState.down_pressed, icon: "down" },
    { label: text.keyTestUp, pressed: keyState.up_pressed, icon: "up" },
    { label: text.keyTestConfirm, pressed: keyState.confirm_pressed, icon: "confirm" },
  ]);
</script>

{#if open}
  <div class="overlay" role="presentation">
    <div class="dialog" role="dialog" aria-modal="true" aria-label={text.keyTestTitle}>
      <h3 class="dialog-title">{text.keyTestTitle}</h3>
      <p class="dialog-desc">{text.keyTestInstruction}</p>

      <div class="keys-row">
        {#each keys as key (key.icon)}
          <div class="key-item">
            <div class="key-icon-wrap" class:pressed={key.pressed}>
              {#if key.icon === "back"}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="32"
                  height="32"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <polyline points="15 18 9 12 15 6"></polyline>
                </svg>
              {:else if key.icon === "down"}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="32"
                  height="32"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <polyline points="6 9 12 15 18 9"></polyline>
                </svg>
              {:else if key.icon === "up"}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="32"
                  height="32"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <polyline points="18 15 12 9 6 15"></polyline>
                </svg>
              {:else if key.icon === "confirm"}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="32"
                  height="32"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
              {/if}
            </div>
            <span class="key-label" class:pressed={key.pressed}>{key.label}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(15, 23, 42, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }

  .dialog {
    width: min(480px, 100%);
    background: #ffffff;
    border-radius: 12px;
    padding: 28px 24px;
    box-shadow: 0 10px 40px rgba(15, 23, 42, 0.25);
  }

  .dialog-title {
    margin: 0 0 8px;
    font-size: 18px;
    font-weight: 600;
    color: #0f172a;
  }

  .dialog-desc {
    margin: 0 0 28px;
    font-size: 14px;
    color: #475569;
  }

  .keys-row {
    display: flex;
    justify-content: center;
    gap: 24px;
  }

  .key-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .key-icon-wrap {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid #cbd5e1;
    background: #f1f5f9;
    color: #94a3b8;
    transition: border-color 0.3s, background 0.3s, color 0.3s;
  }

  .key-icon-wrap.pressed {
    border-color: #22c55e;
    background: #f0fdf4;
    color: #16a34a;
  }

  .key-label {
    font-size: 13px;
    font-weight: 500;
    color: #64748b;
    transition: color 0.3s;
  }

  .key-label.pressed {
    color: #16a34a;
  }
</style>

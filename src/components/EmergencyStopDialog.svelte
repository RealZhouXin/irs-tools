<script lang="ts">
  let {
    open,
    title,
    instruction,
    status,
    showUnlockKeys = false,
    backLabel,
    confirmLabel,
    onRequestClose,
  } = $props<{
    open: boolean;
    title: string;
    instruction: string;
    status: string;
    showUnlockKeys?: boolean;
    backLabel: string;
    confirmLabel: string;
    onRequestClose: () => void;
  }>();

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onRequestClose();
    }
  }
</script>

{#if open}
  <div class="overlay" role="presentation" onclick={handleOverlayClick}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label={title} tabindex="-1">
      <h3 class="dialog-title">{title}</h3>
      <p class="dialog-instruction">{instruction}</p>

      {#if showUnlockKeys}
        <div class="keys-row" aria-label="unlock keys">
          <div class="key-item">
            <div class="key-icon-wrap">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="28"
                height="28"
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
            </div>
            <span class="key-label">{backLabel}</span>
          </div>
          <div class="key-plus">+</div>
          <div class="key-item">
            <div class="key-icon-wrap">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="28"
                height="28"
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
            </div>
            <span class="key-label">{confirmLabel}</span>
          </div>
        </div>
      {/if}

      <p class="dialog-status">{status}</p>
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
    width: min(500px, 100%);
    background: #ffffff;
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 10px 40px rgba(15, 23, 42, 0.25);
  }

  .dialog-title {
    margin: 0 0 10px;
    font-size: 18px;
    font-weight: 600;
    color: #0f172a;
  }

  .dialog-instruction {
    margin: 0 0 16px;
    color: #334155;
    font-size: 14px;
  }

  .keys-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .key-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .key-icon-wrap {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid #2563eb;
    background: #eff6ff;
    color: #1d4ed8;
  }

  .key-label {
    font-size: 13px;
    color: #1e293b;
    font-weight: 500;
  }

  .key-plus {
    font-size: 24px;
    color: #334155;
    font-weight: 700;
  }

  .dialog-status {
    margin: 0;
    color: #64748b;
    font-size: 12px;
  }
</style>

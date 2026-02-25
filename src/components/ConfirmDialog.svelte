<script lang="ts">
  let { open, title, message, yesLabel, noLabel, onYes, onNo } = $props<{
    open: boolean;
    title: string;
    message: string;
    yesLabel: string;
    noLabel: string;
    onYes: () => void;
    onNo: () => void;
  }>();
</script>

{#if open}
  <div class="overlay" role="presentation">
    <div class="dialog" role="dialog" aria-modal="true" aria-label={title}>
      <h3>{title}</h3>
      <p>{message}</p>
      <div class="actions">
        <button class="no" onclick={onNo}>{noLabel}</button>
        <button class="yes" onclick={onYes}>{yesLabel}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(15, 23, 42, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }

  .dialog {
    width: min(420px, 100%);
    background: #ffffff;
    border-radius: 12px;
    padding: 18px;
    box-shadow: 0 10px 30px rgba(15, 23, 42, 0.25);
  }

  .dialog h3 {
    margin: 0 0 10px;
    font-size: 18px;
    color: #0f172a;
  }

  .dialog p {
    margin: 0 0 16px;
    color: #334155;
    font-size: 14px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  button {
    border: none;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: 14px;
    cursor: pointer;
  }

  .no {
    background: #e2e8f0;
    color: #0f172a;
  }

  .yes {
    background: #0284c7;
    color: #ffffff;
  }
</style>


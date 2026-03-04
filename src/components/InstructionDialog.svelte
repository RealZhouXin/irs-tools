<script lang="ts">
  let { open, title, message } = $props<{
    open: boolean;
    title: string;
    message: string;
  }>();
</script>

{#if open}
  <div class="overlay" role="presentation">
    <div class="dialog" role="dialog" aria-modal="true" aria-label={title}>
      <h3>{title}</h3>
      <p>{message}</p>
      <div class="spinner" aria-hidden="true"></div>
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
    margin: 0;
    color: #334155;
    font-size: 14px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    margin: 14px auto 0;
    border: 3px solid #cbd5e1;
    border-top-color: #2563eb;
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

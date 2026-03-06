<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import type { WheelMotorTestPhase } from "../types";

  let {
    open,
    title,
    message,
    phase,
    confirmLabel,
    cancelLabel,
    onConfirm,
    onCancel,
  } = $props<{
    open: boolean;
    title: string;
    message: string;
    phase: WheelMotorTestPhase;
    confirmLabel: string;
    cancelLabel: string;
    onConfirm: () => void;
    onCancel: () => void;
  }>();

  function handleOverlayClick(event: MouseEvent) {
    if (event.target !== event.currentTarget) {
      return;
    }
    if (phase === "lift_confirm") {
      onCancel();
    }
  }
</script>

{#if open}
  <div class="overlay" role="presentation" onclick={handleOverlayClick}>
    <div class="dialog" role="dialog" aria-modal="true" aria-label={title}>
      <h3>{title}</h3>
      <p>{message}</p>
      {#if phase === "lift_confirm"}
        <div class="actions">
          <Button variant="secondary" onclick={onCancel}>{cancelLabel}</Button>
          <Button variant="default" onclick={onConfirm}>{confirmLabel}</Button>
        </div>
      {:else}
        <div class="wheel-wrap" aria-hidden="true">
          <div class="wheel">
            <span class="spoke"></span>
            <span class="spoke"></span>
            <span class="spoke"></span>
            <span class="spoke"></span>
          </div>
        </div>
      {/if}
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

  .wheel-wrap {
    display: flex;
    justify-content: center;
  }

  .wheel {
    width: 88px;
    height: 88px;
    border: 5px solid #0f172a;
    border-radius: 999px;
    position: relative;
    animation: rotate 0.9s linear infinite;
  }

  .spoke {
    position: absolute;
    inset: 50% auto auto 50%;
    width: 2px;
    height: 32px;
    background: #334155;
    transform-origin: top center;
  }

  .spoke:nth-child(1) {
    transform: translate(-50%, -50%) rotate(0deg);
  }

  .spoke:nth-child(2) {
    transform: translate(-50%, -50%) rotate(45deg);
  }

  .spoke:nth-child(3) {
    transform: translate(-50%, -50%) rotate(90deg);
  }

  .spoke:nth-child(4) {
    transform: translate(-50%, -50%) rotate(135deg);
  }

  @keyframes rotate {
    to {
      transform: rotate(360deg);
    }
  }
</style>

<script lang="ts">
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import {
    isEscapeKey,
    isInteractiveDialogTarget,
    isSpaceKey,
  } from "$lib/dialog-shortcuts";

  export type LightColor = "red" | "green" | "blue";
  
  let {
    open,
    title,
    message,
    yesLabel,
    noLabel,
    onYes,
    onNo,
    onRequestClose,
    showLightAnimation = false,
    lightColor = null,
  } = $props<{
    open: boolean;
    title: string;
    message: string;
    yesLabel: string;
    noLabel: string;
    onYes: () => void;
    onNo: () => void;
    onRequestClose: () => void;
    showLightAnimation?: boolean;
    lightColor?: LightColor | null;
  }>();

  let dialogElement = $state<HTMLDivElement | null>(null);
  let wasOpen = false;

  $effect(() => {
    if (open && !wasOpen) {
      void tick().then(() => dialogElement?.focus());
    }
    wasOpen = open;
  });

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onRequestClose();
    }
  }

  function handleDialogKeydown(event: KeyboardEvent) {
    if (isEscapeKey(event)) {
      event.preventDefault();
      onRequestClose();
      return;
    }

    if (isSpaceKey(event) && !isInteractiveDialogTarget(event.target)) {
      event.preventDefault();
      onYes();
    }
  }
</script>

{#if open}
  <div class="overlay" role="presentation" onclick={handleOverlayClick}>
    <div
      bind:this={dialogElement}
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex={-1}
      onkeydown={handleDialogKeydown}
    >
      <h3>{title}</h3>
      <p>{message}</p>
      {#if showLightAnimation && lightColor}
        <div class="light-preview">
          <div class="light-orb" data-color={lightColor} aria-hidden="true"></div>
        </div>
      {/if}
      <div class="actions">
        <Button variant="secondary" onclick={onNo}>{noLabel}</Button>
        <Button variant="default" onclick={onYes}>{yesLabel}</Button>
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

  .light-preview {
    display: flex;
    justify-content: center;
    margin: 0 0 16px;
  }

  .light-orb {
    width: 84px;
    height: 84px;
    border-radius: 999px;
    animation: pulse 1.2s ease-in-out infinite;
  }

  .light-orb[data-color="red"] {
    background: radial-gradient(circle at 35% 35%, #fecaca 0%, #dc2626 60%, #7f1d1d 100%);
    box-shadow: 0 0 22px #ef4444, 0 0 42px rgba(239, 68, 68, 0.55);
  }

  .light-orb[data-color="green"] {
    background: radial-gradient(circle at 35% 35%, #bbf7d0 0%, #16a34a 60%, #14532d 100%);
    box-shadow: 0 0 22px #22c55e, 0 0 42px rgba(34, 197, 94, 0.55);
  }

  .light-orb[data-color="blue"] {
    background: radial-gradient(circle at 35% 35%, #bfdbfe 0%, #2563eb 60%, #1e3a8a 100%);
    box-shadow: 0 0 22px #3b82f6, 0 0 42px rgba(59, 130, 246, 0.55);
  }

  @keyframes pulse {
    0%,
    100% {
      transform: scale(0.96);
      filter: brightness(0.9);
    }
    50% {
      transform: scale(1.06);
      filter: brightness(1.1);
    }
  }
</style>

<script lang="ts">
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import {
    isEscapeKey,
    isInteractiveDialogTarget,
    isSpaceKey,
  } from "$lib/dialog-shortcuts";

  let {
    open,
    title,
    message,
    yesLabel,
    noLabel,
    onYes,
    onNo,
    onRequestClose,
  } = $props<{
    open: boolean;
    title: string;
    message: string;
    yesLabel: string;
    noLabel: string;
    onYes: () => void;
    onNo: () => void;
    onRequestClose: () => void;
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
</style>

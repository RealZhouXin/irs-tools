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
  <div 
    class="fixed inset-0 z-50 bg-zinc-950/40 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-200" 
    role="presentation" 
    onclick={handleOverlayClick}
  >
    <div
      bind:this={dialogElement}
      class="w-full max-w-sm bg-white rounded-2xl shadow-2xl border border-zinc-200/60 p-6 focus:outline-none animate-in zoom-in-95 duration-200 flex flex-col items-center text-center"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onkeydown={handleDialogKeydown}
    >
      <div class="w-16 h-16 rounded-full bg-indigo-50 flex items-center justify-center mb-6 shadow-sm border border-indigo-100">
        <!-- Speaker Icon -->
        <svg class="w-8 h-8 text-indigo-500 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
        </svg>
      </div>

      <h3 class="text-lg font-semibold tracking-tight text-zinc-900 mb-1.5">{title}</h3>
      <p class="text-[14px] text-zinc-500 leading-relaxed font-medium mb-8 max-w-[260px]">{message}</p>

      <div class="flex items-center gap-3 w-full">
        <Button 
          variant="outline" 
          class="flex-1 h-11 rounded-xl text-[14px] font-semibold text-zinc-600 border-zinc-200 hover:bg-zinc-50 hover:text-zinc-900 transition-colors" 
          onclick={onNo}
        >
          {noLabel}
        </Button>
        <Button 
          variant="default" 
          class="flex-1 h-11 rounded-xl text-[14px] font-semibold bg-zinc-900 hover:bg-zinc-800 text-white shadow-md transition-all active:scale-[0.98]" 
          onclick={onYes}
        >
          {yesLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}

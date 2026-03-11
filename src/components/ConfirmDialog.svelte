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
  <!-- overlay -->
  <div 
    class="fixed inset-0 z-50 bg-zinc-950/40 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-200" 
    role="presentation" 
    onclick={handleOverlayClick}
  >
    <!-- dialog -->
    <div
      bind:this={dialogElement}
      class="w-full max-w-sm bg-white rounded-2xl shadow-2xl border border-zinc-200/60 p-6 focus:outline-none animate-in zoom-in-95 duration-200"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onkeydown={handleDialogKeydown}
    >
      <div class="flex flex-col items-center text-center gap-1 mb-6">
        <h3 class="text-lg font-semibold tracking-tight text-zinc-900">{title}</h3>
        <p class="text-[14px] text-zinc-500 leading-relaxed font-medium mt-1">{message}</p>
      </div>

      {#if showLightAnimation && lightColor}
        <div class="flex justify-center mb-8">
          <div class="relative w-20 h-20 rounded-full flex items-center justify-center bg-zinc-50 border border-zinc-100 shadow-inner">
            <div class={
              "absolute w-12 h-12 rounded-full animate-pulse blur-sm opacity-50 " + 
              (lightColor === 'green' ? 'bg-emerald-500' : lightColor === 'red' ? 'bg-rose-500' : 'bg-blue-500')
            }></div>
            <div class={
              "relative w-8 h-8 rounded-full shadow-[0_0_15px_rgba(0,0,0,0.2)] " + 
              (lightColor === 'green' ? 'bg-emerald-500' : lightColor === 'red' ? 'bg-rose-500' : 'bg-blue-500')
            }></div>
          </div>
        </div>
      {/if}

      <div class="flex items-center gap-3 w-full mt-4">
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

<script lang="ts">
  import { tick } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import {
    isEscapeKey,
    isInteractiveDialogTarget,
    isSpaceKey,
  } from "$lib/dialog-shortcuts";
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

  let dialogElement = $state<HTMLDivElement | null>(null);
  let wasOpen = false;

  $effect(() => {
    if (open && !wasOpen) {
      void tick().then(() => dialogElement?.focus());
    }
    wasOpen = open;
  });

  function handleOverlayClick(event: MouseEvent) {
    if (event.target !== event.currentTarget) {
      return;
    }
    if (phase === "lift_confirm") {
      onCancel();
    }
  }

  function handleDialogKeydown(event: KeyboardEvent) {
    if (phase !== "lift_confirm") {
      return;
    }

    if (isEscapeKey(event)) {
      event.preventDefault();
      onCancel();
      return;
    }

    if (isSpaceKey(event) && !isInteractiveDialogTarget(event.target)) {
      event.preventDefault();
      onConfirm();
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
      class="w-full max-w-md bg-white rounded-2xl shadow-2xl border border-zinc-200/60 p-8 focus:outline-none animate-in zoom-in-95 duration-200 flex flex-col items-center text-center"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      onkeydown={handleDialogKeydown}
    >
      <div class="mb-6 flex flex-col items-center gap-1.5">
        <h3 class="text-xl font-semibold tracking-tight text-zinc-900">{title}</h3>
        <p class="text-[14px] text-zinc-500 max-w-[280px] leading-relaxed font-medium">{message}</p>
      </div>

      {#if phase === "lift_confirm"}
        <div class="w-24 h-24 mb-6 rounded-full bg-amber-50 flex items-center justify-center border-2 border-amber-200/60 shadow-sm relative">
           <svg class="w-10 h-10 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 10l7-7m0 0l7 7m-7-7v18" />
           </svg>
           <span class="absolute -bottom-1 text-[10px] font-bold text-amber-600 bg-amber-100 px-2 py-0.5 rounded uppercase tracking-widest border border-amber-200/80">LIFT UP</span>
        </div>
        <div class="flex items-center gap-3 w-full mt-2">
          <Button 
            variant="outline" 
            class="flex-1 h-11 rounded-xl text-[14px] font-semibold text-zinc-600 border-zinc-200 hover:bg-zinc-50 transition-colors" 
            onclick={onCancel}
          >
            {cancelLabel}
          </Button>
          <Button 
            variant="default" 
            class="flex-1 h-11 rounded-xl text-[14px] font-semibold bg-zinc-900 hover:bg-zinc-800 text-white shadow-md transition-transform active:scale-[0.98]" 
            onclick={onConfirm}
          >
            {confirmLabel}
          </Button>
        </div>
      {:else}
        <div class="w-full flex justify-center py-6">
          <div class="wheel relative w-24 h-24 border-8 border-zinc-800 rounded-full shadow-[0_0_20px_rgba(0,0,0,0.1)]">
            <span class="spoke absolute left-1/2 top-1/2 w-1 h-9 bg-zinc-300 origin-top -translate-x-1/2 -translate-y-1/2 rotate-0"></span>
            <span class="spoke absolute left-1/2 top-1/2 w-1 h-9 bg-zinc-300 origin-top -translate-x-1/2 -translate-y-1/2 rotate-45"></span>
            <span class="spoke absolute left-1/2 top-1/2 w-1 h-9 bg-zinc-300 origin-top -translate-x-1/2 -translate-y-1/2 rotate-90"></span>
            <span class="spoke absolute left-1/2 top-1/2 w-1 h-9 bg-zinc-300 origin-top -translate-x-1/2 -translate-y-1/2 rotate-[135deg]"></span>
            <div class="absolute left-1/2 top-1/2 w-3 h-3 bg-zinc-800 rounded-full -translate-x-1/2 -translate-y-1/2"></div>
          </div>
        </div>
        <div class="flex items-center gap-2 mt-4 text-[13px] font-medium text-emerald-600 bg-emerald-50 px-4 py-1.5 rounded-full border border-emerald-200/60 shadow-sm animate-pulse">
          <svg class="animate-spin -ml-1 mr-1 h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>
          Testing in progress...
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .wheel {
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>

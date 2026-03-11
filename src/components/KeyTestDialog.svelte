<script lang="ts">
  import type { KeyStatePayload, Translation } from "../types";

  let {
    open,
    text,
    keyState,
    onRequestClose,
  } = $props<{
    open: boolean;
    text: Translation;
    keyState: KeyStatePayload;
    onRequestClose: () => void;
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

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onRequestClose();
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
      class="w-full max-w-lg bg-white rounded-2xl shadow-2xl border border-zinc-200/60 p-8 focus:outline-none animate-in zoom-in-95 duration-200"
      role="dialog"
      aria-modal="true"
      aria-label={text.keyTestTitle}
      tabindex="-1"
    >
      <div class="flex flex-col items-center text-center gap-1.5 mb-10 mt-2">
        <h3 class="text-xl font-semibold tracking-tight text-zinc-900">{text.keyTestTitle}</h3>
        <p class="text-[14px] text-zinc-500 font-medium leading-relaxed max-w-sm">{text.keyTestInstruction}</p>
      </div>

      <div class="flex justify-center gap-6 sm:gap-8 mb-4">
        {#each keys as key (key.icon)}
          <div class="flex flex-col items-center gap-4">
            <div 
              class={"w-20 h-20 rounded-full flex items-center justify-center border-2 transition-all duration-300 shadow-sm " + 
              (key.pressed 
                ? "border-emerald-500 bg-emerald-50 text-emerald-600 scale-105 shadow-[0_0_20px_rgba(16,185,129,0.2)]" 
                : "border-zinc-200 bg-zinc-50/50 text-zinc-400")}
            >
              {#if key.icon === "back"}
                <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
              {:else if key.icon === "down"}
                <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
              {:else if key.icon === "up"}
                <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 15 12 9 6 15"></polyline></svg>
              {:else if key.icon === "confirm"}
                <svg class="w-8 h-8" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
              {/if}
            </div>
            <span class={"text-[13px] font-bold tracking-wide uppercase transition-colors duration-300 " + (key.pressed ? "text-emerald-600" : "text-zinc-400")}>
              {key.label}
            </span>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

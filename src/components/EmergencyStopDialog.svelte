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
  <div 
    class="fixed inset-0 z-50 bg-rose-950/40 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in duration-200" 
    role="presentation" 
    onclick={handleOverlayClick}
  >
    <div 
      class="w-full max-w-lg bg-white rounded-2xl shadow-[0_20px_50px_rgba(225,29,72,0.2)] border-2 border-rose-200/50 p-8 focus:outline-none animate-in zoom-in-95 duration-200 flex flex-col items-center text-center relative overflow-hidden" 
      role="dialog" 
      aria-modal="true" 
      aria-label={title} 
      tabindex="-1"
    >
      <div class="absolute inset-x-0 top-0 h-1.5 bg-rose-500"></div>
      
      <div class="w-20 h-20 mb-6 rounded-full bg-rose-50 flex items-center justify-center border-2 border-rose-100 relative">
        <div class="absolute inset-0 rounded-full animate-ping bg-rose-400 opacity-20"></div>
        <svg class="w-10 h-10 text-rose-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>

      <h3 class="text-xl font-bold tracking-tight text-zinc-900 mb-2">{title}</h3>
      <p class="text-[14px] text-zinc-600 font-medium leading-relaxed max-w-sm mb-6">{instruction}</p>

      {#if showUnlockKeys}
        <div class="flex items-center justify-center gap-4 mb-6 bg-zinc-50 py-4 px-6 rounded-xl border border-zinc-100/80 w-full" aria-label="unlock keys">
          <div class="flex flex-col items-center gap-2">
            <div class="w-14 h-14 rounded-full flex items-center justify-center border-2 border-indigo-200 bg-indigo-50 text-indigo-500 shadow-sm">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
            </div>
            <span class="text-[12px] font-bold text-zinc-500 tracking-wide uppercase">{backLabel}</span>
          </div>
          
          <div class="text-xl font-bold text-zinc-300">+</div>
          
          <div class="flex flex-col items-center gap-2">
            <div class="w-14 h-14 rounded-full flex items-center justify-center border-2 border-indigo-200 bg-indigo-50 text-indigo-500 shadow-sm">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
            </div>
            <span class="text-[12px] font-bold text-zinc-500 tracking-wide uppercase">{confirmLabel}</span>
          </div>
        </div>
      {/if}

      <div class="inline-flex items-center gap-2 text-[13px] font-semibold text-rose-600 bg-rose-50 px-3 py-1.5 rounded-md border border-rose-200/50">
        <span class="relative flex h-2 w-2">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-rose-400 opacity-75"></span>
          <span class="relative inline-flex rounded-full h-2 w-2 bg-rose-500"></span>
        </span>
        {status}
      </div>
    </div>
  </div>
{/if}

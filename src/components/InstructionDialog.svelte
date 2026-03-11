<script lang="ts">
  let { open, title, message, onRequestClose } = $props<{
    open: boolean;
    title: string;
    message: string;
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
    class="fixed inset-0 z-50 bg-zinc-950/40 backdrop-blur-sm flex items-center justify-center p-4 animate-in fade-in duration-200" 
    role="presentation" 
    onclick={handleOverlayClick}
  >
    <div 
      class="w-full max-w-sm bg-white rounded-2xl shadow-2xl border border-zinc-200/60 p-6 focus:outline-none animate-in zoom-in-95 duration-200 flex flex-col items-center text-center" 
      role="dialog" 
      aria-modal="true" 
      aria-label={title}
    >
      <div class="w-14 h-14 rounded-full bg-blue-50 flex items-center justify-center border border-blue-100 shadow-sm mb-4 relative">
        <div class="absolute inset-0 rounded-full border-[3px] border-blue-100"></div>
        <div class="absolute inset-0 rounded-full border-[3px] border-blue-500 border-t-transparent animate-spin"></div>
        <svg class="w-6 h-6 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
      </div>
      
      <h3 class="text-lg font-semibold tracking-tight text-zinc-900 mb-1.5">{title}</h3>
      <p class="text-[14px] text-zinc-500 leading-relaxed font-medium">{message}</p>
    </div>
  </div>
{/if}

<script lang="ts">
  import type { StatusKey, SummaryState, Translation } from "../types";

  let { text, statusKey, summaryState, summaryLabel, machineSn } = $props<{
    text: Translation;
    statusKey: StatusKey;
    summaryState: SummaryState;
    summaryLabel: string;
    machineSn: string | null;
  }>();

  // Define styling dynamically based on status
  const getStateStyles = () => {
    switch (summaryState) {
      case "pass":
        return {
          bg: "bg-emerald-50 border-emerald-200/60",
          text: "text-emerald-700",
          icon: "text-emerald-500",
          indicator: "bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]"
        };
      case "fail":
        return {
          bg: "bg-rose-50 border-rose-200/60",
          text: "text-rose-700",
          icon: "text-rose-500",
          indicator: "bg-rose-500 animate-pulse shadow-[0_0_8px_rgba(244,63,94,0.5)]"
        };
      case "pending":
        return {
          bg: "bg-amber-50 border-amber-200/60",
          text: "text-amber-700",
          icon: "text-amber-500",
          indicator: "bg-amber-500 animate-ping"
        };
      default:
        return {
          bg: "bg-zinc-50 border-zinc-200/60",
          text: "text-zinc-500",
          icon: "text-zinc-400",
          indicator: "bg-zinc-300"
        };
    }
  };

  const styles = $derived(getStateStyles());
</script>

<div class="flex flex-col gap-3">
  <!-- 核心数据卡片面板 -->
  <div class="rounded-xl border shadow-sm p-4 flex flex-col gap-4 transition-colors duration-300 {styles.bg}">
    
    <!-- 状态行 -->
    <div class="flex items-start justify-between">
      <div class="flex flex-col gap-1">
        <span class="text-[11px] font-bold uppercase tracking-widest text-zinc-500/80">
          {text.statusTitle}
        </span>
        <div class="flex items-center gap-2 mt-0.5">
          <svg class="w-5 h-5 {styles.icon}" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            {#if summaryState === 'pass'}
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            {:else if summaryState === 'fail'}
              <path stroke-linecap="round" stroke-linejoin="round" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
            {:else if summaryState === 'pending'}
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            {:else}
              <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            {/if}
          </svg>
          <span class="text-[15px] font-semibold tracking-tight text-zinc-900">
            {text.status[statusKey]}
          </span>
        </div>
      </div>

      <!-- 最终结论指示标 -->
      <div class="flex flex-col items-end gap-1.5">
        <span class="text-[10px] font-bold uppercase tracking-widest text-zinc-400">
          {text.table.result}
        </span>
        <div class="flex items-center gap-2 h-7 px-3 bg-white/60 rounded-md border border-white/40 shadow-[0_1px_2px_rgba(0,0,0,0.02)] backdrop-blur-sm">
          <span class="flex h-2 w-2 rounded-full {styles.indicator}"></span>
          <span class="text-sm font-bold tracking-wide {styles.text} uppercase">
            {summaryLabel}
          </span>
        </div>
      </div>
    </div>

    <!-- 分割线 -->
    <div class="h-px w-full bg-zinc-200/50"></div>

    <!-- 底部辅助信息 (SN) -->
    <div class="flex items-center justify-between">
      <span class="text-xs font-medium text-zinc-500">{text.machineSnLabel}</span>
      <span class="text-xs font-mono font-semibold text-zinc-700 bg-white/50 px-1.5 py-0.5 rounded border border-zinc-200/50">
        {machineSn ?? "WAITING..."}
      </span>
    </div>

  </div>
</div>

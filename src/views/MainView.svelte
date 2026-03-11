<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import ResultsTable from "../components/ResultsTable.svelte";
  import StatusCard from "../components/StatusCard.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import type {
    SummaryState,
    StatusKey,
    TestResult,
    Translation,
    Language,
  } from "../types";

  const ALL_STAGES_VALUE = "__all__";

  let {
    text,
    language,
    results,
    error,
    running,
    statusKey,
    summaryState,
    summaryLabel,
    machineSn,
    retesting,
    exportError,
    exportSuccess,
    exporting,
    stopping,
    stageOptions,
    selectedStage,
    onStart,
    onStop,
    onOpenExport,
    onRetest,
    onSelectStage,
    onToggleLanguage,
  } = $props<{
    text: Translation;
    language: Language;
    results: TestResult[];
    error: string | null;
    running: boolean;
    statusKey: StatusKey;
    summaryState: SummaryState;
    summaryLabel: string;
    machineSn: string | null;
    retesting: string | null;
    exportError: string | null;
    exportSuccess: string | null;
    exporting: boolean;
    stopping: boolean;
    stageOptions: string[];
    selectedStage: string;
    onStart: () => void;
    onStop: () => void;
    onOpenExport: () => void;
    onRetest: (groupName: string) => void;
    onSelectStage: (stage: string) => void;
    onToggleLanguage: () => void;
  }>();
</script>

<div class="flex h-screen w-full overflow-hidden bg-zinc-50/50">
  <!-- 左侧控制面板 (Sidebar) -->
  <aside
    class="w-[340px] flex flex-col border-r border-zinc-200/80 bg-white shadow-[1px_0_12px_rgba(0,0,0,0.02)] shrink-0 z-10 relative"
  >
    <!-- 头部区: 标题与设备状态 -->
    <div class="p-6 border-b border-zinc-100 flex flex-col gap-1">
      <h1
        class="text-xl font-semibold tracking-tight text-zinc-900 leading-none"
      >
        {text.title}
      </h1>
      <p class="text-[13px] font-medium text-zinc-500 mt-1">{text.subtitle}</p>

      <div
        class="mt-5 flex items-center gap-2.5 px-3 py-2.5 bg-zinc-50/80 rounded-lg border border-zinc-200/60 shadow-sm"
      >
        <span class="relative flex h-2.5 w-2.5 ml-0.5">
          {#if running}
            <span
              class="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75"
            ></span>
            <span
              class="relative inline-flex rounded-full h-2.5 w-2.5 bg-amber-500"
            ></span>
          {:else if machineSn}
            <span
              class="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]"
            ></span>
          {:else}
            <span
              class="relative inline-flex rounded-full h-2.5 w-2.5 bg-zinc-300"
            ></span>
          {/if}
        </span>
        <span
          class="text-[13px] font-mono font-semibold tracking-tight text-zinc-700 truncate"
        >
          {machineSn ? machineSn : "No Device Connected"}
        </span>
      </div>
    </div>

    <!-- 测试控制区 -->
    <div
      class="p-6 flex flex-col gap-6 flex-1 overflow-y-auto custom-scrollbar"
    >
      <div class="flex flex-col gap-2.5">
        <label
          for="stage-select"
          class="text-[11px] font-bold text-zinc-400 uppercase tracking-widest"
          >{text.stageLabel}</label
        >
        <Select.Root
          type="single"
          value={selectedStage}
          onValueChange={(v) => onSelectStage(v)}
          disabled={running}
        >
          <Select.Trigger
            class="h-10 w-full flex items-center justify-between rounded-lg border border-zinc-200/80 bg-white px-3 py-1 text-[13px] font-medium text-zinc-800 shadow-sm transition-all hover:border-zinc-300 focus:outline-none focus:ring-2 focus:ring-zinc-900/10 focus:border-zinc-400 disabled:cursor-not-allowed disabled:bg-zinc-50 disabled:text-zinc-400"
          >
            {selectedStage === ALL_STAGES_VALUE ? text.stageAll : selectedStage}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value={ALL_STAGES_VALUE} label={text.stageAll} />
            {#each stageOptions as stage (stage)}
              <Select.Item value={stage} label={stage} />
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <div class="flex flex-col gap-3">
        {#if !running}
          <Button
            class="w-full h-12 rounded-lg text-sm font-semibold transition-all bg-zinc-900 text-white hover:bg-zinc-800 shadow-md ring-1 ring-zinc-950/50"
            onclick={onStart}
          >
            <svg
              class="mr-2 h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2.5"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
              /><path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              /></svg
            >
            {text.start}
          </Button>
        {:else}
          <Button
            variant="destructive"
            class="w-full h-12 rounded-lg text-sm font-semibold transition-all shadow-md {stopping
              ? 'opacity-80'
              : ''}"
            onclick={onStop}
            disabled={stopping}
          >
            {#if stopping}
              <svg
                class="animate-spin mr-2 h-4 w-4"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                ><circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                ></circle><path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                ></path></svg
              >
              {text.stopping}
            {:else}
              <svg
                class="mr-2 h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="2.5"
                ><path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                /><path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z"
                /></svg
              >
              {text.stop}
            {/if}
          </Button>
        {/if}
      </div>

      <!-- 状态指示面板 (包裹原有的StatusCard使其融入侧边栏风格) -->
      <div class="mt-4 border-t border-zinc-100/80 pt-6">
        <p
          class="text-[11px] font-bold text-zinc-400 uppercase tracking-widest block mb-3"
          >Status Overview</p
        >
        <div
          class="[&>div]:shadow-none [&>div]:border-zinc-200/60 [&>div]:bg-zinc-50/50"
        >
          <StatusCard
            {text}
            {statusKey}
            {summaryState}
            {summaryLabel}
            {machineSn}
          />
        </div>
      </div>
    </div>
  </aside>

  <!-- 右侧主工作区 (Main Workspace) -->
  <main class="flex-1 flex flex-col h-screen min-w-0 bg-[#F9FAFB]">
    <!-- 顶部工具栏 (Toolbar) -->
    <header
      class="h-16 flex items-center justify-between px-8 border-b border-zinc-200/60 bg-white/70 backdrop-blur-md shrink-0 z-10 sticky top-0"
    >
      <div class="flex items-center gap-4">
        {#if exportSuccess}
          <span
            class="text-[13px] font-medium text-emerald-700 bg-emerald-50 px-3 py-1.5 rounded-md border border-emerald-200/60 flex items-center shadow-sm animate-in fade-in slide-in-from-bottom-2"
          >
            <svg
              class="mr-1.5 h-3.5 w-3.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="3"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M5 13l4 4L19 7"
              ></path></svg
            >
            {exportSuccess}
          </span>
        {/if}
        {#if exportError}
          <span
            class="text-[13px] font-medium text-rose-700 bg-rose-50 px-3 py-1.5 rounded-md border border-rose-200/60 flex items-center shadow-sm animate-in fade-in slide-in-from-bottom-2"
          >
            <svg
              class="mr-1.5 h-3.5 w-3.5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2.5"
              ><path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              ></path></svg
            >
            {exportError}
          </span>
        {/if}
      </div>

      <div class="flex items-center gap-3">
        <Button
          variant="outline"
          size="sm"
          onclick={onOpenExport}
          disabled={running || exporting}
          class="h-9 px-3.5 text-[13px] font-medium text-zinc-600 bg-white hover:bg-zinc-50 border-zinc-200 shadow-sm transition-all focus-visible:ring-1 focus-visible:ring-zinc-300"
        >
          {#if exporting}
            <svg
              class="animate-spin mr-2 h-3.5 w-3.5"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              ><circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
              ></circle><path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              ></path></svg
            >
          {:else}
            <svg
              class="mr-2 h-3.5 w-3.5"
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><path
                d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
              ></path><polyline points="14 2 14 8 20 8"></polyline><line
                x1="12"
                y1="18"
                x2="12"
                y2="12"
              ></line><line x1="9" y1="15" x2="12" y2="18"></line><line
                x1="15"
                y1="15"
                x2="12"
                y2="18"
              ></line></svg
            >
          {/if}
          {exporting ? text.exporting : text.export}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={onToggleLanguage}
          class="h-9 px-3.5 text-[13px] font-medium text-zinc-500 hover:text-zinc-900 border border-transparent hover:bg-zinc-100/80 transition-all"
        >
          <svg
            class="mr-2 h-3.5 w-3.5"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><circle cx="12" cy="12" r="10"></circle><line
              x1="2"
              y1="12"
              x2="22"
              y2="12"
            ></line><path
              d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
            ></path></svg
          >
          {text.langLabel}
        </Button>
      </div>
    </header>

    <!-- 测试数据表区 -->
    <div class="flex-1 overflow-hidden p-6 relative flex flex-col">
      <div
        class="flex-1 overflow-hidden rounded-xl border border-zinc-200/80 bg-white shadow-sm flex flex-col relative w-full"
      >
        <!-- 内部绝对定位撑开以允许完全接管滚动 -->
        <div class="absolute inset-0 overflow-auto">
          <ResultsTable
            {results}
            {text}
            {language}
            {error}
            {running}
            {summaryState}
            {retesting}
            {onRetest}
            class="min-w-full"
          />
        </div>
      </div>

      <!-- 底部微提示 -->
      <footer class="mt-4 px-1 flex flex-col gap-1 shrink-0">
        <p class="text-[11px] text-zinc-400 font-medium">
          {text.configPrefix}
          <code
            class="px-1.5 py-0.5 bg-zinc-200/50 rounded text-zinc-500 font-mono tracking-tight"
            >AppData\Roaming\com.greenworks.irs-tools\config\threshold.toml</code
          >
          /
          <code
            class="px-1.5 py-0.5 bg-zinc-200/50 rounded text-zinc-500 font-mono tracking-tight"
            >tests.yaml</code
          >
          {text.configSuffix}
        </p>
      </footer>
    </div>
  </main>
</div>

<style>
  /* 隐藏边栏极小滚动条，保持整洁，但支持滚动 */
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #e4e4e7; /* oklch(0.92 0 0) - zinc 200 */
    border-radius: 4px;
  }
  .custom-scrollbar:hover::-webkit-scrollbar-thumb {
    background: #d4d4d8; /* oklch(0.87 0 0) - zinc 300 */
  }
</style>



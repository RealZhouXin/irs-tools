<script lang="ts">
  import ResultsTable from "../components/ResultsTable.svelte";
  import StatusCard from "../components/StatusCard.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import type {
    SummaryState,
    StatusKey,
    TestResult,
    Translation,
  } from "../types";

  const ALL_STAGES_VALUE = "__all__";

  let {
    text,
    results,
    error,
    running,
    statusKey,
    summaryState,
    summaryLabel,
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
    results: TestResult[];
    error: string | null;
    running: boolean;
    statusKey: StatusKey;
    summaryState: SummaryState;
    summaryLabel: string;
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

<main class="max-w-5xl mx-auto p-8 flex flex-col gap-6 w-full lg:px-5 lg:py-6">
  <header class="flex justify-between items-center gap-4 flex-wrap">
    <div>
      <h1>{text.title}</h1>
      <p class="subtitle">{text.subtitle}</p>
    </div>
    <div class="flex items-center gap-3 flex-wrap lg:justify-end">
      <div class="flex items-center gap-2">
        <label for="stage-select" class="text-sm text-slate-600 font-semibold">{text.stageLabel}</label>
        <select
          id="stage-select"
          class="min-w-[160px] h-9 rounded-md border border-slate-200 bg-white px-3 py-1 text-sm shadow-sm transition-colors cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
          value={selectedStage}
          onchange={(event) =>
            onSelectStage((event.currentTarget as HTMLSelectElement).value)}
          disabled={running}
        >
          <option value={ALL_STAGES_VALUE}>{text.stageAll}</option>
          {#each stageOptions as stage (stage)}
            <option value={stage}>{stage}</option>
          {/each}
        </select>
      </div>
      <Button variant="default" onclick={onStart} disabled={running}>
        {text.start}
      </Button>
      <Button variant="secondary" onclick={onStop} disabled={!running || stopping}>
        {stopping ? text.stopping : text.stop}
      </Button>
      <Button variant="outline" onclick={onOpenExport} disabled={running || exporting}>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
          class="mr-2"
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="12" y1="18" x2="12" y2="12"></line>
          <line x1="9" y1="15" x2="12" y2="18"></line>
          <line x1="15" y1="15" x2="12" y2="18"></line>
        </svg>
        {exporting ? text.exporting : text.export}
      </Button>
      <Button variant="ghost" onclick={onToggleLanguage}>
        {text.langLabel}
      </Button>
    </div>
  </header>

  <StatusCard
    {text}
    {statusKey}
    {summaryState}
    {summaryLabel}
  />

  {#if exportSuccess}
    <div class="success">{exportSuccess}</div>
  {/if}
  {#if exportError}
    <div class="error">{exportError}</div>
  {/if}

  <ResultsTable
    {results}
    {text}
    {error}
    {running}
    {summaryState}
    {retesting}
    onRetest={onRetest}
  />

  <section class="note">
    <h3>{text.configTitle}</h3>
    <p>
      {text.configPrefix}
      <code>AppData\Roaming\com.greenworks.irs-tools\config\threshold.toml</code>
      {text.configMiddle}
      <code>AppData\Roaming\com.greenworks.irs-tools\config\tests.toml</code>
      {text.configSuffix}
    </p>
  </section>
</main>

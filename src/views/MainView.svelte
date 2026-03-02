<script lang="ts">
  import ResultsTable from "../components/ResultsTable.svelte";
  import StatusCard from "../components/StatusCard.svelte";
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
    stageOptions,
    selectedStage,
    onStart,
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
    stageOptions: string[];
    selectedStage: string;
    onStart: () => void;
    onOpenExport: () => void;
    onRetest: (groupName: string) => void;
    onSelectStage: (stage: string) => void;
    onToggleLanguage: () => void;
  }>();
</script>

<main class="container">
  <header>
    <div>
      <h1>{text.title}</h1>
      <p class="subtitle">{text.subtitle}</p>
    </div>
    <div class="header-actions">
      <div class="stage-filter">
        <label for="stage-select">{text.stageLabel}</label>
        <select
          id="stage-select"
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
      <button class="primary" onclick={onStart} disabled={running}>
        {text.start}
      </button>
      <button class="secondary" onclick={onOpenExport} disabled={running || exporting}>
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
        >
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="12" y1="18" x2="12" y2="12"></line>
          <line x1="9" y1="15" x2="12" y2="18"></line>
          <line x1="15" y1="15" x2="12" y2="18"></line>
        </svg>
        {exporting ? text.exporting : text.export}
      </button>
      <button class="lang-toggle" onclick={onToggleLanguage}>
        {text.langLabel}
      </button>
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

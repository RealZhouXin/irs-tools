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
    stageOptions,
    selectedStage,
    onStart,
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
    stageOptions: string[];
    selectedStage: string;
    onStart: () => void;
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

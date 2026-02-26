<script lang="ts">
  import ResultsTable from "../components/ResultsTable.svelte";
  import StatusCard from "../components/StatusCard.svelte";
  import type {
    SummaryState,
    StatusKey,
    TestResult,
    Translation,
  } from "../types";

  let {
    text,
    results,
    error,
    running,
    statusKey,
    summaryState,
    summaryLabel,
    retesting,
    onStart,
    onRetest,
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
    onStart: () => void;
    onRetest: (groupName: string) => void;
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

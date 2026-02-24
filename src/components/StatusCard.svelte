<script lang="ts">
  import type { StatusKey, SummaryState, Translation } from "../types";

  let { text, statusKey, summaryState, summaryLabel } = $props<{
    text: Translation;
    statusKey: StatusKey;
    summaryState: SummaryState;
    summaryLabel: string;
  }>();
</script>

<section class="status">
  <div>
    <h2>{text.statusTitle}</h2>
    <p>{text.status[statusKey]}</p>
  </div>
  <div
    class="summary"
    data-state={summaryState === "idle" ? undefined : summaryState}
  >
    {summaryLabel}
  </div>
</section>

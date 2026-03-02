<script lang="ts">
  import type { StatusKey, SummaryState, Translation } from "../types";
  import * as Card from "$lib/components/ui/card/index.js";

  let { text, statusKey, summaryState, summaryLabel } = $props<{
    text: Translation;
    statusKey: StatusKey;
    summaryState: SummaryState;
    summaryLabel: string;
  }>();

  // 根据当前状态返回对应的 Tailwind 颜色类名
  const getStateColorClass = () => {
    switch (summaryState) {
      case "pass":
        return "text-green-600 font-bold";
      case "fail":
        return "text-destructive font-bold";
      case "pending":
        return "text-amber-500 font-bold animate-pulse";
      default:
        return "text-muted-foreground font-medium";
    }
  };
</script>

<Card.Root>
  <Card.Header class="flex flex-row items-center justify-between pb-2 space-y-0">
    <div>
      <Card.Title class="text-sm font-medium">{text.statusTitle}</Card.Title>
      <Card.Description class="mt-1 text-2xl font-bold text-foreground">
        {text.status[statusKey]}
      </Card.Description>
    </div>
    <div class="flex flex-col items-end">
      <span class="text-sm text-muted-foreground">{text.table.result}:</span>
      <span class="text-xl {getStateColorClass()}">
        {summaryLabel}
      </span>
    </div>
  </Card.Header>
</Card.Root>


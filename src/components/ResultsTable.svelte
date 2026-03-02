<script lang="ts">
  import type { TestResult, SummaryState, Translation } from "../types";
  import * as Card from "$lib/components/ui/card/index.js";
  import * as Table from "$lib/components/ui/table/index.js";
  import { Button } from "$lib/components/ui/button/index.js";

  type StageGroup = {
    stage: string;
    items: TestResult[];
    passed: boolean;
  };

  // Svelte 5 Props using $props() rune
  let { results, text, error, running, summaryState, retesting, onRetest } =
    $props<{
      results: TestResult[];
      text: Translation;
      error: string | null;
      running: boolean;
      summaryState: SummaryState;
      retesting: string | null;
      onRetest: (groupName: string) => void;
    }>();

  const stageGroups = $derived.by<StageGroup[]>(() => {
    const groups = new Map<string, TestResult[]>();

    for (const result of results) {
      const stageName =
        result.stage.trim().length > 0 ? result.stage.trim() : text.stageUnassigned;
      const existing = groups.get(stageName);
      if (existing) {
        existing.push(result);
      } else {
        groups.set(stageName, [result]);
      }
    }

    return Array.from(groups, ([stage, items]) => ({
      stage,
      items,
      passed: items.every((item) => item.passed),
    }));
  });
</script>

<section class="space-y-6">
  {#if error}
    <div class="rounded-md bg-destructive/15 p-4 py-3 text-sm text-destructive font-medium border border-destructive/20 shadow-sm">{error}</div>
  {/if}

  {#if results.length === 0}
    <Card.Root>
      <div class="rounded-md border-0">
        <Table.Root>
          <Table.Header>
            <Table.Row>
              <Table.Head>{text.table.group}</Table.Head>
              <Table.Head>{text.table.command}</Table.Head>
              <Table.Head>{text.table.range}</Table.Head>
              <Table.Head>{text.table.value}</Table.Head>
              <Table.Head class="w-[180px]">{text.table.result}</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            <Table.Row>
              <Table.Cell colspan={5} class="h-24 text-center text-muted-foreground">
                {text.table.empty}
              </Table.Cell>
            </Table.Row>
          </Table.Body>
        </Table.Root>
      </div>
    </Card.Root>
  {:else}
    <div class="flex flex-col gap-6">
      {#each stageGroups as stageGroup (stageGroup.stage)}
        <Card.Root>
          <Card.Header class="flex flex-row items-center justify-between pb-2">
            <Card.Title class="text-base">{stageGroup.stage}</Card.Title>
            <span class="text-sm font-bold {stageGroup.passed ? 'text-green-600' : 'text-destructive'}">
              {stageGroup.passed ? text.pass : text.fail}
            </span>
          </Card.Header>

          <div class="border-t">
            <Table.Root>
              <Table.Header>
                <Table.Row class="bg-muted/50 hover:bg-muted/50">
                  <Table.Head class="w-[260px] font-semibold">{text.table.group}</Table.Head>
                  <Table.Head class="font-semibold">{text.table.command}</Table.Head>
                  <Table.Head class="font-semibold">{text.table.range}</Table.Head>
                  <Table.Head class="font-semibold">{text.table.value}</Table.Head>
                  <Table.Head class="w-[200px] font-semibold">{text.table.result}</Table.Head>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {#each stageGroup.items as group (stageGroup.stage + "-" + group.name)}
                  <Table.Row class="hover:bg-transparent bg-background">
                    <Table.Cell class="font-medium">{group.name}</Table.Cell>
                    <Table.Cell>
                      <code class="relative rounded bg-muted px-[0.4rem] py-[0.2rem] font-mono text-sm font-semibold">{group.command}</code>
                    </Table.Cell>
                    <Table.Cell class="text-muted-foreground">-</Table.Cell>
                    <Table.Cell class="text-muted-foreground">-</Table.Cell>
                    <Table.Cell>
                      <div class="flex items-center gap-3">
                        <span class="font-medium w-10 {group.passed ? 'text-green-600' : 'text-destructive'}">
                          {group.passed ? text.pass : text.fail}
                        </span>
                        <Button
                          variant="outline"
                          size="sm"
                          class="h-7 text-xs px-2"
                          onclick={() => onRetest(group.name)}
                          disabled={running ||
                            retesting !== null ||
                            summaryState === "pending" ||
                            summaryState === "idle"}
                        >
                          {retesting === group.name ? text.retesting : text.retest}
                        </Button>
                      </div>
                    </Table.Cell>
                  </Table.Row>

                  {#each group.checks as check (stageGroup.stage + "-" + group.name + "-" + check.name)}
                    <Table.Row class="bg-muted/20 border-b-0 last:border-b">
                      <Table.Cell class="pl-8 text-muted-foreground text-sm flex items-center gap-2">
                        <span class="w-1 h-1 rounded-full bg-slate-300"></span>
                        {check.name}
                      </Table.Cell>
                      <Table.Cell class="text-muted-foreground shrink-0">-</Table.Cell>
                      <Table.Cell class="text-slate-600 font-mono text-xs">
                        {check.min === null || check.max === null
                          ? "-"
                          : `${check.min} ~ ${check.max}`}
                      </Table.Cell>
                      <Table.Cell class="font-mono font-medium text-sm">
                        {check.value === null ? "-" : check.value}
                      </Table.Cell>
                      <Table.Cell>
                        <span class="text-sm font-medium {check.passed ? 'text-green-600' : 'text-destructive'}">
                          {check.passed ? text.pass : text.fail}
                        </span>
                      </Table.Cell>
                    </Table.Row>
                  {/each}
                {/each}
              </Table.Body>
            </Table.Root>
          </div>
        </Card.Root>
      {/each}
    </div>
  {/if}
</section>


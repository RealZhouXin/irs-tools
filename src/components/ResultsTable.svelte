<script lang="ts">
  import type { TestResult, SummaryState, Translation } from "../types";

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

<section>
  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if results.length === 0}
    <table>
      <thead>
        <tr>
          <th>{text.table.group}</th>
          <th>{text.table.command}</th>
          <th>{text.table.range}</th>
          <th>{text.table.value}</th>
          <th>{text.table.result}</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td colspan="5" class="empty">
            {text.table.empty}
          </td>
        </tr>
      </tbody>
    </table>
  {:else}
    <div class="stage-cards">
      {#each stageGroups as stageGroup (stageGroup.stage)}
        <article class="stage-card">
          <header class="stage-card-header">
            <h3>{stageGroup.stage}</h3>
            <span class={stageGroup.passed ? "pass" : "fail"}>
              {stageGroup.passed ? text.pass : text.fail}
            </span>
          </header>

          <table>
            <thead>
              <tr>
                <th>{text.table.group}</th>
                <th>{text.table.command}</th>
                <th>{text.table.range}</th>
                <th>{text.table.value}</th>
                <th>{text.table.result}</th>
              </tr>
            </thead>
            <tbody>
              {#each stageGroup.items as group (stageGroup.stage + "-" + group.name)}
                <tr class="group-row">
                  <td>{group.name}</td>
                  <td>
                    <code>{group.command}</code>
                  </td>
                  <td>-</td>
                  <td>-</td>
                  <td class={group.passed ? "pass" : "fail"}>
                    <div class="group-actions">
                      <span>{group.passed ? text.pass : text.fail}</span>
                      <button
                        class="retest"
                        onclick={() => onRetest(group.name)}
                        disabled={running ||
                          retesting !== null ||
                          summaryState === "pending" ||
                          summaryState === "idle"}
                      >
                        {retesting === group.name ? text.retesting : text.retest}
                      </button>
                    </div>
                  </td>
                </tr>

                {#each group.checks as check (stageGroup.stage + "-" + group.name + "-" + check.name)}
                  <tr class="child-row">
                    <td class="indent">{check.name}</td>
                    <td>-</td>
                    <td>
                      {check.min === null || check.max === null
                        ? "-"
                        : `${check.min} ~ ${check.max}`}
                    </td>
                    <td>{check.value === null ? "-" : check.value}</td>
                    <td class={check.passed ? "pass" : "fail"}>
                      {check.passed ? text.pass : text.fail}
                    </td>
                  </tr>
                {/each}
              {/each}
            </tbody>
          </table>
        </article>
      {/each}
    </div>
  {/if}
</section>

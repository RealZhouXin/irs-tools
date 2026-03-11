<script lang="ts">
  import type { TestResult, SummaryState, Translation, Language } from "../types";
  import { Button } from "$lib/components/ui/button/index.js";

  type StageGroup = {
    stage: string;
    items: TestResult[];
    passed: boolean;
  };

  let { results, text, language, error, running, summaryState, retesting, onRetest } =
    $props<{
      results: TestResult[];
      text: Translation;
      language: Language;
      error: string | null;
      running: boolean;
      summaryState: SummaryState;
      retesting: string | null;
      onRetest: (groupName: string) => void;
      class?: string;
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

  function getVersionFromRawResponse(group: TestResult): string {
    const prefix = "Version=";
    return group.raw_response.startsWith(prefix)
      ? group.raw_response.slice(prefix.length)
      : group.raw_response;
  }

  function getCheckDisplayValue(group: TestResult, check: TestResult["checks"][number]): string {
    if (check.display_value) {
      return check.display_value;
    }
    if (group.command === "ParamId798" && check.name === "version_not_empty") {
      return getVersionFromRawResponse(group);
    }
    return check.value === null ? "-" : String(check.value);
  }

  function getCheckDisplayRange(check: TestResult["checks"][number]): string {
    if (check.display_min && check.display_max) {
      return `${check.display_min} ~ ${check.display_max}`;
    }
    return check.min === null || check.max === null ? "-" : `${check.min} ~ ${check.max}`;
  }

  function getGroupName(group: TestResult): string {
    return group.names?.[language] ?? group.name;
  }
</script>

<div class="h-full flex flex-col w-full">
  {#if error}
    <div class="m-6 rounded-lg bg-rose-50 p-4 border border-rose-200/60 shadow-sm flex items-start gap-3 text-rose-700 animate-in slide-in-from-top-2">
      <svg class="h-5 w-5 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path></svg>
      <div class="text-[13px] font-medium leading-relaxed">{error}</div>
    </div>
  {/if}

  <div class="w-full relative">
    <table class="w-full text-left border-collapse border-spacing-0">
      <!-- 固定的表头增强表格滚动体验 -->
      <thead class="sticky top-0 z-10 bg-white/95 backdrop-blur-sm shadow-[0_1px_rgba(0,0,0,0.06)]">
        <tr>
          <th class="h-10 px-6 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wider whitespace-nowrap">{text.table.group}</th>
          <th class="px-6 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wider whitespace-nowrap">{text.table.command}</th>
          <th class="px-6 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wider whitespace-nowrap">{text.table.range}</th>
          <th class="px-6 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wider whitespace-nowrap">{text.table.value}</th>
          <th class="px-6 py-2 text-xs font-semibold text-zinc-500 uppercase tracking-wider whitespace-nowrap w-[180px]">{text.table.result}</th>
        </tr>
      </thead>

      <tbody class="text-[13px]">
        {#if results.length === 0}
          <tr>
            <td colspan="5" class="h-48 text-center bg-zinc-50/50">
              <div class="flex flex-col items-center justify-center text-zinc-400">
                <svg class="h-10 w-10 mb-3 opacity-60" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1"><path stroke-linecap="round" stroke-linejoin="round" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
                <p class="font-medium text-sm">{text.table.empty}</p>
                <p class="text-xs mt-1">Press Start to begin testing sequence</p>
              </div>
            </td>
          </tr>
        {:else}
          {#each stageGroups as stageGroup (stageGroup.stage)}
            <!-- 分组标题层 -->
            <tr class="group text-zinc-900 bg-zinc-50/80 hover:bg-zinc-100/50 transition-colors border-b border-zinc-100">
              <td colspan="5" class="px-6 py-3">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <svg class="w-4 h-4 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
                    <span class="font-semibold text-[14px]">{stageGroup.stage}</span>
                  </div>
                  {#if stageGroup.passed}
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-bold bg-emerald-100/80 text-emerald-700 tracking-widest uppercase shadow-sm">STAGE PASS</span>
                  {:else}
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-bold bg-rose-100/80 text-rose-700 tracking-widest uppercase shadow-sm">STAGE FAIL</span>
                  {/if}
                </div>
              </td>
            </tr>

            <!-- 测试项目 -->
            {#each stageGroup.items as group (stageGroup.stage + "-" + group.name)}
              <tr class="border-b border-zinc-100/70 hover:bg-zinc-50 transition-colors group/item">
                <!-- Group Name -->
                <td class="px-6 py-3">
                  <div class="font-medium text-zinc-800">{getGroupName(group)}</div>
                </td>
                
                <!-- Command Box -->
                <td class="px-6 py-3">
                  <span class="inline-flex items-center px-2 py-1 rounded-md bg-zinc-100 border border-zinc-200 text-zinc-600 font-mono text-[11px] font-semibold tracking-tight shadow-sm">
                    {group.command}
                  </span>
                </td>

                <td class="px-6 py-3 text-zinc-400 font-mono">-</td>
                <td class="px-6 py-3 text-zinc-400 font-mono">-</td>
                
                <!-- Result & Retest -->
                <td class="px-6 py-3">
                  <div class="flex items-center justify-between gap-3">
                    {#if group.passed}
                      <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-semibold bg-emerald-50 text-emerald-700 border border-emerald-200/60 shadow-sm">
                        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> PASS
                      </span>
                    {:else}
                      <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-semibold bg-rose-50 text-rose-700 border border-rose-200/60 shadow-sm animate-pulse">
                        <span class="w-1.5 h-1.5 rounded-full bg-rose-500"></span> FAIL
                      </span>
                    {/if}
                    
                    <Button
                      variant="outline"
                      size="sm"
                      class="h-7 text-[11px] px-2.5 bg-white border-zinc-200 text-zinc-600 hover:text-zinc-900 hover:bg-zinc-50 shadow-sm opacity-0 group-hover/item:opacity-100 transition-opacity focus-visible:opacity-100 disabled:opacity-50"
                      onclick={() => onRetest(group.name)}
                      disabled={running || retesting !== null || summaryState === "pending" || summaryState === "idle"}
                    >
                      {#if retesting === group.name}
                        <svg class="animate-spin -ml-0.5 mr-1.5 h-3 w-3" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>
                        {text.retesting}
                      {:else}
                        <svg class="-ml-0.5 mr-1 h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path></svg>
                        {text.retest}
                      {/if}
                    </Button>
                  </div>
                </td>
              </tr>

              <!-- 展开的 Checks -->
              {#each group.checks as check (stageGroup.stage + "-" + group.name + "-" + check.name)}
                <tr class="group/check bg-white hover:bg-zinc-50/50 transition-colors border-b border-zinc-100/50 last:border-zinc-200">
                  <td class="px-6 py-2.5 pl-10">
                    <div class="flex items-center gap-2.5 text-zinc-500">
                      <div class="w-[3px] h-[3px] rounded-full bg-zinc-300"></div>
                      <span class="text-[12.5px]">{check.name}</span>
                    </div>
                  </td>
                  <td class="px-6 py-2.5 text-zinc-300">-</td>
                  <td class="px-6 py-2.5">
                    <span class="font-mono text-xs text-zinc-500 bg-zinc-50 border border-zinc-100 rounded px-1.5 py-0.5">
                      {getCheckDisplayRange(check)}
                    </span>
                  </td>
                  <td class="px-6 py-2.5">
                    <span class="font-mono text-xs font-semibold {check.passed ? 'text-zinc-800' : 'text-rose-600'}">
                      {getCheckDisplayValue(group, check)}
                    </span>
                  </td>
                  <td class="px-6 py-2.5">
                    {#if check.passed}
                      <span class="text-[11px] font-bold text-emerald-600 uppercase tracking-widest">PASS</span>
                    {:else}
                      <span class="text-[11px] font-bold text-rose-600 uppercase tracking-widest">FAIL</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            {/each}
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>

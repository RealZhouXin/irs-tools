<script lang="ts">
    import type { TestResult, SummaryState, Translation } from "../types";

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
</script>

<section>
    {#if error}
        <div class="error">{error}</div>
    {/if}

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
            {#if results.length === 0}
                <tr>
                    <td colspan="5" class="empty">
                        {text.table.empty}
                    </td>
                </tr>
            {:else}
                {#each results as group (group.name)}
                    <tr class="group-row">
                        <td>{group.name}</td>
                        <td>
                            <code>{group.command}</code>
                        </td>
                        <td>-</td>
                        <td>-</td>
                        <td class={group.passed ? "pass" : "fail"}>
                            <div class="group-actions">
                                <span
                                    >{group.passed
                                        ? text.pass
                                        : text.fail}</span
                                >
                                <button
                                    class="retest"
                                    onclick={() => onRetest(group.name)}
                                    disabled={running ||
                                        retesting !== null ||
                                        summaryState === "pending" ||
                                        summaryState === "idle"}
                                >
                                    {retesting === group.name
                                        ? text.retesting
                                        : text.retest}
                                </button>
                            </div>
                        </td>
                    </tr>
                    {#each group.checks as check (group.name + "-" + check.name)}
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
            {/if}
        </tbody>
    </table>
</section>

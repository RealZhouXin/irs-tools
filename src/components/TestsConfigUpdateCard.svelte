<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import type { TestsConfigUpdateStatus, Translation } from "../types";

  let {
    text,
    status,
    statusError,
    actionMessage,
    actionError,
    applying,
    ignoring,
    actionDisabled,
    onApply,
    onIgnore,
  } = $props<{
    text: Translation;
    status: TestsConfigUpdateStatus | null;
    statusError: string | null;
    actionMessage: string | null;
    actionError: string | null;
    applying: boolean;
    ignoring: boolean;
    actionDisabled: boolean;
    onApply: () => void;
    onIgnore: () => void;
  }>();

  let showPaths = $state(false);

  function showPendingNotice() {
    return status?.newDefaultAvailable === true && !status.ignoredPendingDefault;
  }

  function showIgnoredNotice() {
    return status?.newDefaultAvailable === true && status.ignoredPendingDefault;
  }
</script>

<div class="bg-white border border-zinc-200/60 shadow-sm rounded-xl overflow-hidden relative">
  <div class="absolute top-0 left-0 w-full h-24 bg-gradient-to-br from-amber-50 via-white to-zinc-50 -z-0"></div>

  <div class="p-6 relative z-10 space-y-4">
    <div class="flex items-start justify-between gap-4">
      <div class="space-y-1">
        <h3 class="text-lg font-semibold tracking-tight text-zinc-900">
          {text.testsConfigTitle}
        </h3>
        <p class="text-[13px] leading-relaxed text-zinc-500">
          {text.testsConfigSubtitle}
        </p>
      </div>
      {#if status?.newDefaultAvailable}
        <span
          class={`shrink-0 rounded-full px-3 py-1 text-[11px] font-semibold uppercase tracking-wider ${
            status.ignoredPendingDefault
              ? "bg-zinc-100 text-zinc-600"
              : "bg-amber-100 text-amber-700"
          }`}
        >
          {status.ignoredPendingDefault ? "ignored" : "pending"}
        </span>
      {/if}
    </div>

    {#if statusError}
      <div class="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
        {text.testsConfigLoadError}: {statusError}
      </div>
    {:else if !status}
      <div class="py-8 flex items-center justify-center text-sm text-zinc-500">
        <div class="h-4 w-4 rounded-full border-2 border-zinc-300 border-t-zinc-900 animate-spin"></div>
        <span class="ml-3">{text.settingsLoading}</span>
      </div>
    {:else}
      {#if showPendingNotice()}
        <div class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          {text.testsConfigUpdateAvailable}
        </div>
      {:else if showIgnoredNotice()}
        <div class="rounded-lg border border-zinc-200 bg-zinc-50 px-4 py-3 text-sm text-zinc-700">
          {text.testsConfigUpdateIgnored}
        </div>
      {:else}
        <div class="rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          {text.testsConfigNoUpdate}
        </div>
      {/if}

      <div class="grid gap-3 rounded-xl border border-zinc-100 bg-zinc-50/70 p-4 md:grid-cols-2">
        <div class="space-y-1">
          <div class="text-[12px] font-semibold uppercase tracking-wider text-zinc-400">
            {text.testsConfigCurrentPath}
          </div>
          <div class="text-sm font-medium text-zinc-800">
            {status.localIsModified
              ? text.testsConfigLocalModified
              : text.testsConfigUsingDefault}
          </div>
        </div>
        <div class="space-y-1">
          <div class="text-[12px] font-semibold uppercase tracking-wider text-zinc-400">
            {text.testsConfigPendingVersion}
          </div>
          <div class="text-sm font-mono text-zinc-700">
            {status.pendingDefaultVersion ?? "-"}
          </div>
        </div>
      </div>

      <div class="flex flex-wrap gap-3">
        <Button
          variant="outline"
          size="sm"
          class="border-zinc-200 hover:bg-zinc-50 text-zinc-700 h-9 transition-all"
          onclick={() => (showPaths = !showPaths)}
        >
          {showPaths ? text.testsConfigHidePaths : text.testsConfigShowPaths}
        </Button>

        <Button
          size="sm"
          class="h-9 bg-zinc-900 text-white hover:bg-zinc-800"
          onclick={onApply}
          disabled={!status.newDefaultAvailable || applying || actionDisabled}
        >
          {#if applying}
            <div class="h-3.5 w-3.5 rounded-full border-2 border-zinc-300 border-t-white animate-spin mr-2"></div>
          {/if}
          {text.testsConfigApply}
        </Button>

        {#if status.newDefaultAvailable && !status.ignoredPendingDefault}
          <Button
            variant="ghost"
            size="sm"
            class="h-9 text-zinc-600 hover:text-zinc-900 hover:bg-zinc-100"
            onclick={onIgnore}
            disabled={ignoring || actionDisabled}
          >
            {#if ignoring}
              <div class="h-3.5 w-3.5 rounded-full border-2 border-zinc-300 border-t-zinc-700 animate-spin mr-2"></div>
            {/if}
            {text.testsConfigIgnore}
          </Button>
        {/if}
      </div>

      {#if showPaths}
        <div class="space-y-3 rounded-xl border border-zinc-200 bg-white p-4">
          <div class="space-y-1">
            <div class="text-[12px] font-semibold uppercase tracking-wider text-zinc-400">
              {text.testsConfigCurrentPath}
            </div>
            <code class="block break-all rounded-lg bg-zinc-50 px-3 py-2 text-[12px] text-zinc-700">
              {status.activePath}
            </code>
          </div>

          {#if status.pendingDefaultPath}
            <div class="space-y-1">
              <div class="text-[12px] font-semibold uppercase tracking-wider text-zinc-400">
                {text.testsConfigPendingPath}
              </div>
              <code class="block break-all rounded-lg bg-zinc-50 px-3 py-2 text-[12px] text-zinc-700">
                {status.pendingDefaultPath}
              </code>
            </div>
          {/if}
        </div>
      {/if}

      {#if actionMessage}
        <div class="rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700">
          {actionMessage}
        </div>
      {/if}

      {#if actionError}
        <div class="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
          {actionError}
        </div>
      {/if}

      {#if actionDisabled}
        <p class="rounded-lg border border-zinc-100 bg-zinc-50 px-3 py-2 text-xs text-zinc-500">
          {text.testsConfigActionDisabled}
        </p>
      {/if}
    {/if}
  </div>
</div>

<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import type {
    AppUpdateInfo,
    AppUpdateProgress,
    AppUpdateStatus,
    Translation,
  } from "../types";

  let {
    text,
    aboutError,
    appName,
    appVersion,
    tauriVersion,
    updateSupported,
    updateStatus,
    updateInfo,
    updateErrorMessage,
    updateProgress,
    updateActionDisabled,
    onCheckUpdate,
    onInstallUpdate,
  } = $props<{
    text: Translation;
    aboutError: string | null;
    appName: string | null;
    appVersion: string | null;
    tauriVersion: string | null;
    updateSupported: boolean;
    updateStatus: AppUpdateStatus;
    updateInfo: AppUpdateInfo | null;
    updateErrorMessage: string | null;
    updateProgress: AppUpdateProgress | null;
    updateActionDisabled: boolean;
    onCheckUpdate: () => void;
    onInstallUpdate: () => void;
  }>();

  function formatDate(date: string | null) {
    if (!date) {
      return "-";
    }

    const parsed = new Date(date);
    return Number.isNaN(parsed.getTime()) ? date : parsed.toLocaleString();
  }

  function getLatestVersionLabel() {
    if (updateInfo) {
      return updateInfo.version;
    }
    if (updateStatus === "up_to_date") {
      return appVersion ?? "-";
    }
    return text.updateNotChecked;
  }

  function getUpdateStatusLabel() {
    switch (updateStatus) {
      case "checking":
        return text.updateChecking;
      case "available":
        return text.updateReady;
      case "up_to_date":
        return text.updateUpToDate;
      case "downloading":
        return text.updateDownloading;
      case "installing":
        return text.updateInstalling;
      case "error":
        return updateErrorMessage
          ? `${text.updateError}: ${updateErrorMessage}`
          : text.updateError;
      default:
        return text.updateNotChecked;
    }
  }

  function getProgressPercent() {
    if (!updateProgress || updateProgress.contentLength === null || updateProgress.contentLength <= 0) {
      return null;
    }

    return Math.min(
      100,
      Math.round((updateProgress.downloaded / updateProgress.contentLength) * 100),
    );
  }

  function getActionLabel() {
    if (updateStatus === "available") {
      return text.updateInstallButton;
    }
    if (updateStatus === "checking") {
      return text.updateChecking;
    }
    if (updateStatus === "downloading") {
      return text.updateDownloading;
    }
    if (updateStatus === "installing") {
      return text.updateInstalling;
    }
    return text.updateCheckButton;
  }

  function handleAction() {
    if (updateStatus === "available") {
      onInstallUpdate();
      return;
    }

    onCheckUpdate();
  }
</script>

<div class="bg-white border border-zinc-200/60 shadow-sm rounded-xl overflow-hidden relative">
  <!-- 装饰性背景 -->
  <div class="absolute top-0 left-0 w-full h-32 bg-gradient-to-br from-zinc-100 to-white -z-0"></div>

  <div class="p-6 relative z-10">
    {#if aboutError}
      <div class="p-4 bg-rose-50/50 border border-rose-100 rounded-lg text-sm text-rose-600 flex items-center gap-3">
        <svg class="h-5 w-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
        <span>{text.aboutError}: {aboutError}</span>
      </div>
    {:else if !appVersion}
      <div class="py-10 flex justify-center items-center">
        <div class="h-5 w-5 rounded-full border-2 border-zinc-300 border-t-zinc-900 animate-spin"></div>
        <span class="ml-3 text-sm text-zinc-500">{text.aboutLoading}</span>
      </div>
    {:else}
      <!-- App Info Section -->
      <div class="flex items-start gap-5">
        <div class="h-16 w-16 bg-white border border-zinc-200/60 shadow-sm rounded-2xl flex items-center justify-center shrink-0">
          <svg class="h-8 w-8 text-zinc-900" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
        </div>
        <div class="flex flex-col gap-1 pt-1">
          <h3 class="text-xl font-bold tracking-tight text-zinc-900">{appName ?? "-"}</h3>
          <p class="text-[13px] font-medium text-zinc-500">{text.aboutVersion}: <span class="text-zinc-700">{appVersion}</span></p>
          <p class="text-[13px] font-medium text-zinc-400 mt-0.5">{text.aboutTauriVersion}: {tauriVersion ?? "-"}</p>
        </div>
      </div>

      <!-- Divider -->
      <div class="h-px bg-zinc-100 my-8"></div>

      <!-- Updater Section -->
      <div class="space-y-6">
        <div class="flex items-center justify-between">
          <div class="flex flex-col gap-1">
            <h4 class="text-sm font-semibold text-zinc-800 uppercase tracking-wider">{text.updateTitle}</h4>
            <div class="flex items-center gap-2">
              {#if updateStatus === "checking" || updateStatus === "downloading" || updateStatus === "installing"}
                <div class="h-2 w-2 rounded-full bg-blue-500 animate-pulse"></div>
              {:else if updateStatus === "available"}
                <div class="h-2 w-2 rounded-full bg-amber-500"></div>
              {:else if updateStatus === "up_to_date"}
                <div class="h-2 w-2 rounded-full bg-emerald-500"></div>
              {:else if updateStatus === "error"}
                <div class="h-2 w-2 rounded-full bg-rose-500"></div>
              {:else}
                <div class="h-2 w-2 rounded-full bg-zinc-300"></div>
              {/if}
              <span class="text-[13px] font-medium text-zinc-500">
                {getUpdateStatusLabel()}
              </span>
            </div>
          </div>

          {#if updateSupported}
            <Button
              variant="outline"
              size="sm"
              class="border-zinc-200 hover:bg-zinc-50 text-zinc-700 h-9 transition-all"
              onclick={handleAction}
              disabled={updateActionDisabled || updateStatus === "checking" || updateStatus === "downloading" || updateStatus === "installing"}
            >
              {#if updateStatus === "checking" || updateStatus === "downloading" || updateStatus === "installing"}
                <div class="h-3.5 w-3.5 rounded-full border-2 border-zinc-400 border-t-zinc-700 animate-spin mr-2"></div>
              {/if}
              {getActionLabel()}
            </Button>
          {/if}
        </div>

        <div class="grid grid-cols-2 gap-4 bg-zinc-50/50 rounded-xl p-4 border border-zinc-100">
          <div class="flex flex-col gap-1">
            <span class="text-[12px] font-semibold text-zinc-400 uppercase tracking-wider">{text.updateCurrentVersion}</span>
            <span class="text-sm font-medium text-zinc-800 font-mono">{appVersion ?? updateInfo?.currentVersion ?? "-"}</span>
          </div>
          <div class="flex flex-col gap-1">
            <span class="text-[12px] font-semibold text-zinc-400 uppercase tracking-wider">{text.updateLatestVersion}</span>
            <span class="text-sm font-medium text-zinc-800 font-mono">{getLatestVersionLabel()}</span>
          </div>
        </div>

        {#if updateInfo?.date}
        <div class="flex items-center gap-2 text-[13px] text-zinc-500">
          <svg class="h-4 w-4 shrink-0 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
          {text.updateReleaseDate}: <span class="text-zinc-700">{formatDate(updateInfo.date)}</span>
        </div>
        {/if}

        {#if updateInfo?.body}
          <div class="bg-zinc-50 border border-zinc-100 rounded-lg p-4 max-h-32 overflow-y-auto custom-scrollbar">
            <p class="text-[13px] leading-relaxed text-zinc-600 whitespace-pre-wrap">{updateInfo.body}</p>
          </div>
        {/if}

        {#if updateProgress}
          <div class="space-y-2 pt-2">
            <div class="h-1.5 w-full bg-zinc-100 rounded-full overflow-hidden">
              <div
                class="h-full bg-zinc-900 transition-all duration-300 ease-out"
                style={`width: ${getProgressPercent() ?? 100}%`}
              ></div>
            </div>
            <div class="flex justify-between items-center text-xs font-medium text-zinc-500">
              <span>
                {#if updateProgress.phase === "installing"}
                  {text.updateInstalling}
                {:else}
                  {text.updateDownloading}
                {/if}
              </span>
              {#if getProgressPercent() !== null}
                <span>{getProgressPercent()}%</span>
              {/if}
            </div>
          </div>
        {/if}

        {#if !updateSupported}
          <p class="text-sm font-medium text-amber-600 bg-amber-50 border border-amber-100 px-3 py-2 rounded-md">
            {text.updateDevOnly}
          </p>
        {:else if updateActionDisabled}
          <p class="text-xs text-zinc-400 bg-zinc-50 border border-zinc-100 px-3 py-2 rounded-md">
            {text.updateDisabledDuringTest}
          </p>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: #e4e4e7;
    border-radius: 4px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: #d4d4d8;
  }
</style>

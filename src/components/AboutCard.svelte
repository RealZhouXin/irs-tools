<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
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

<Card.Root>
  <Card.Header>
    <Card.Title>{text.aboutTitle}</Card.Title>
    <Card.Description>{text.aboutSubtitle}</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if aboutError}
      <p class="text-sm font-medium text-destructive">
        {text.aboutError}: {aboutError}
      </p>
    {:else if !appVersion}
      <p class="text-sm text-muted-foreground">{text.aboutLoading}</p>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-6 pt-2">
        <div class="flex flex-col space-y-1">
          <span class="text-sm font-medium text-muted-foreground">{text.aboutName}</span>
          <span class="text-sm font-bold text-foreground">{appName ?? "-"}</span>
        </div>
        <div class="flex flex-col space-y-1">
          <span class="text-sm font-medium text-muted-foreground">{text.aboutVersion}</span>
          <span class="text-sm font-bold text-foreground">{appVersion}</span>
        </div>
        <div class="flex flex-col space-y-1">
          <span class="text-sm font-medium text-muted-foreground">{text.aboutTauriVersion}</span>
          <span class="text-sm font-bold text-foreground">{tauriVersion ?? "-"}</span>
        </div>
      </div>

      <div class="mt-6 border-t border-border pt-6 space-y-4">
        <div class="flex flex-col gap-1">
          <h4 class="text-base font-semibold text-foreground">{text.updateTitle}</h4>
          <p class="text-sm text-muted-foreground">{getUpdateStatusLabel()}</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-6">
          <div class="flex flex-col space-y-1">
            <span class="text-sm font-medium text-muted-foreground">{text.updateCurrentVersion}</span>
            <span class="text-sm font-bold text-foreground">{appVersion ?? updateInfo?.currentVersion ?? "-"}</span>
          </div>
          <div class="flex flex-col space-y-1">
            <span class="text-sm font-medium text-muted-foreground">{text.updateLatestVersion}</span>
            <span class="text-sm font-bold text-foreground">{getLatestVersionLabel()}</span>
          </div>
          <div class="flex flex-col space-y-1">
            <span class="text-sm font-medium text-muted-foreground">{text.updateReleaseDate}</span>
            <span class="text-sm font-bold text-foreground">{formatDate(updateInfo?.date ?? null)}</span>
          </div>
        </div>

        {#if updateInfo?.body}
          <p class="text-sm text-muted-foreground whitespace-pre-wrap">{updateInfo.body}</p>
        {/if}

        {#if updateProgress}
          <div class="space-y-2">
            <div class="h-2 rounded-full bg-muted overflow-hidden">
              <div
                class="h-full bg-primary transition-all"
                style={`width: ${getProgressPercent() ?? 100}%`}
              ></div>
            </div>
            <p class="text-xs text-muted-foreground">
              {#if updateProgress.phase === "installing"}
                {text.updateInstalling}
              {:else if getProgressPercent() !== null}
                {text.updateDownloading} ({getProgressPercent()}%)
              {:else}
                {text.updateDownloading}
              {/if}
            </p>
          </div>
        {/if}

        {#if !updateSupported}
          <p class="text-sm text-muted-foreground">{text.updateDevOnly}</p>
        {:else}
          <div class="flex flex-col items-start gap-2">
            <Button
              variant="default"
              onclick={handleAction}
              disabled={updateActionDisabled || updateStatus === "checking" || updateStatus === "downloading" || updateStatus === "installing"}
            >
              {getActionLabel()}
            </Button>
            {#if updateActionDisabled}
              <p class="text-xs text-muted-foreground">{text.updateDisabledDuringTest}</p>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </Card.Content>
</Card.Root>

<script lang="ts">
  import AboutCard from "../components/AboutCard.svelte";
  import SettingsForm from "../components/SettingsForm.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import type {
    AppUpdateInfo,
    AppUpdateProgress,
    AppUpdateStatus,
    Translation,
  } from "../types";

  type SettingsDraft = {
    mode: "network" | "serial";
    ip_address: string;
    port: string;
    port_number: number;
    read_timeout_ms: number;
    log_level: "error" | "warn" | "info" | "debug" | "trace";
  };

  let {
    text,
    settingsDraft,
    settingsSaving,
    settingsSaved,
    settingsError,
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
    onToggleLanguage,
    onSave,
    onCheckUpdate,
    onInstallUpdate,
  } = $props<{
    text: Translation;
    settingsDraft: SettingsDraft | null;
    settingsSaving: boolean;
    settingsSaved: boolean;
    settingsError: string | null;
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
    onToggleLanguage: () => void;
    onSave: () => void;
    onCheckUpdate: () => void;
    onInstallUpdate: () => void;
  }>();
</script>

<div class="h-screen w-full bg-zinc-50/50 flex flex-col md:flex-row overflow-hidden relative">
  <!-- 桌面感设置左侧导航区 (Sidebar style list) 预留，当前为单页面设计，保持占位一致性。 -->
  <aside class="w-72 flex flex-col border-r border-zinc-200/60 bg-white/70 backdrop-blur-xl shrink-0 p-6 z-10">
    <div class="mb-8">
      <h1 class="text-2xl font-bold tracking-tight text-zinc-900">{text.settingsTitle}</h1>
      <p class="text-[13px] font-medium text-zinc-500 mt-1">{text.settingsSubtitle}</p>
    </div>

    <nav class="flex flex-col gap-1.5 flex-1">
      <a href="#" class="px-4 py-2.5 bg-zinc-100/80 text-zinc-900 font-medium text-[14px] rounded-lg shadow-sm border border-zinc-200/50 flex items-center gap-3 relative transition-all">
        <div class="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-5 bg-zinc-900 rounded-r-full"></div>
        <svg class="w-4 h-4 text-zinc-700" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
        General
      </a>
      <a href="#" class="px-4 py-2.5 text-zinc-500 hover:text-zinc-900 font-medium text-[14px] rounded-lg hover:bg-zinc-100/50 flex items-center gap-3 transition-colors pointer-events-none opacity-50">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 002-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path></svg>
        Test Presets
      </a>
    </nav>
    
    <div class="mt-auto border-t border-zinc-200/50 pt-4">
      <Button variant="ghost" class="w-full justify-start text-zinc-500 hover:text-zinc-900" onclick={onToggleLanguage}>
        <svg class="mr-2 h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
        {text.langLabel}
      </Button>
    </div>
  </aside>

  <!-- 右侧表单设置滚动区 -->
  <main class="flex-1 overflow-x-hidden overflow-y-auto no-scrollbar pb-16">
    <div class="max-w-3xl mx-auto p-8 pt-10 flex flex-col gap-10">
      
      <!-- 配置板块区 -->
      <section class="flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div class="flex flex-col gap-1 border-b border-zinc-200/60 pb-3">
          <h2 class="text-lg font-semibold tracking-tight text-zinc-900 flex items-center gap-2">
            <svg class="h-5 w-5 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"></path></svg>
            System Parameters
          </h2>
        </div>
        <SettingsForm
          {text}
          {settingsDraft}
          {settingsSaving}
          {settingsSaved}
          {settingsError}
          onSave={onSave}
        />
      </section>

      <!-- 关于与更新板块区 -->
      <section class="flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-3 duration-500">
        <div class="flex flex-col gap-1 border-b border-zinc-200/60 pb-3">
          <h2 class="text-lg font-semibold tracking-tight text-zinc-900 flex items-center gap-2">
            <svg class="h-5 w-5 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            Updates & System Info
          </h2>
        </div>
        <AboutCard
          {text}
          {aboutError}
          {appName}
          {appVersion}
          {tauriVersion}
          {updateSupported}
          {updateStatus}
          {updateInfo}
          {updateErrorMessage}
          {updateProgress}
          {updateActionDisabled}
          {onCheckUpdate}
          {onInstallUpdate}
        />
      </section>
      
    </div>
  </main>
</div>

<style>
  /* 完全隐藏右侧滚动条营造沉浸感，但保留滚动功能 */
  .no-scrollbar::-webkit-scrollbar {
    width: 0px;
    background: transparent;
  }
  .no-scrollbar {
    scrollbar-width: none;
    -ms-overflow-style: none;
  }
</style>

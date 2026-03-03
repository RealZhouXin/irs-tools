<script lang="ts">
  import AboutCard from "../components/AboutCard.svelte";
  import SettingsForm from "../components/SettingsForm.svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import type { Translation } from "../types";

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
    onToggleLanguage,
    onSave,
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
    onToggleLanguage: () => void;
    onSave: () => void;
  }>();
</script>

<section class="max-w-4xl mx-auto p-8 flex flex-col gap-6 w-full lg:px-5 lg:py-6">
  <header class="flex justify-between items-center gap-4">
    <div>
      <h1>{text.settingsTitle}</h1>
      <p class="subtitle">{text.settingsSubtitle}</p>
    </div>
    <div class="flex items-center gap-3">
      <Button variant="ghost" onclick={onToggleLanguage}>
        {text.langLabel}
      </Button>
    </div>
  </header>

  <SettingsForm
    {text}
    {settingsDraft}
    {settingsSaving}
    {settingsSaved}
    {settingsError}
    onSave={onSave}
  />

  <AboutCard
    {text}
    {aboutError}
    {appName}
    {appVersion}
    {tauriVersion}
  />
</section>

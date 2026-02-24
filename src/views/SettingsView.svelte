<script lang="ts">
  import AboutCard from "../components/AboutCard.svelte";
  import SettingsForm from "../components/SettingsForm.svelte";
  import type { Translation } from "../types";

  type SettingsDraft = {
    mode: "network" | "serial";
    ip_address: string;
    port: string;
    port_number: number;
    read_timeout_ms: number;
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

<section class="settings">
  <header class="settings-header">
    <div>
      <h1>{text.settingsTitle}</h1>
      <p class="subtitle">{text.settingsSubtitle}</p>
    </div>
    <div class="header-actions">
      <button class="lang-toggle" onclick={onToggleLanguage}>
        {text.langLabel}
      </button>
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

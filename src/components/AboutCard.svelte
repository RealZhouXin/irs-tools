<script lang="ts">
  import type { Translation } from "../types";

  let { text, aboutError, appName, appVersion, tauriVersion } = $props<{
    text: Translation;
    aboutError: string | null;
    appName: string | null;
    appVersion: string | null;
    tauriVersion: string | null;
  }>();
</script>

<div class="about-card">
  <div>
    <h2>{text.aboutTitle}</h2>
    <p class="subtitle">{text.aboutSubtitle}</p>
  </div>
  {#if aboutError}
    <p class="settings-error">
      {text.aboutError}: {aboutError}
    </p>
  {:else if !appVersion}
    <p class="settings-message">{text.aboutLoading}</p>
  {:else}
    <div class="about-grid">
      <div class="about-item">
        <span class="about-label">{text.aboutName}</span>
        <span class="about-value">{appName ?? "-"}</span>
      </div>
      <div class="about-item">
        <span class="about-label">{text.aboutVersion}</span>
        <span class="about-value">{appVersion}</span>
      </div>
      <div class="about-item">
        <span class="about-label">{text.aboutTauriVersion}</span>
        <span class="about-value">{tauriVersion ?? "-"}</span>
      </div>
    </div>
  {/if}
</div>

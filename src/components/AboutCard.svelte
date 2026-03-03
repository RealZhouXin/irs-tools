<script lang="ts">
  import type { Translation } from "../types";
  import * as Card from "$lib/components/ui/card/index.js";

  let { text, aboutError, appName, appVersion, tauriVersion } = $props<{
    text: Translation;
    aboutError: string | null;
    appName: string | null;
    appVersion: string | null;
    tauriVersion: string | null;
  }>();
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
    {/if}
  </Card.Content>
</Card.Root>


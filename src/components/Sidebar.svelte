<script lang="ts">
  import type { Translation } from "../types";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  let { view, text, onOpenTests, onOpenSettings } = $props<{
    view: "main" | "settings";
    text: Translation;
    onOpenTests: () => void;
    onOpenSettings: () => void;
  }>();
</script>

<aside class="w-[68px] sm:w-[60px] bg-background flex flex-col items-center justify-between py-4 h-screen shrink-0 border-r">
  <div class="flex flex-col gap-3 w-full px-2 items-center">
    <Tooltip.Provider>
      <Tooltip.Root>
        <Tooltip.Trigger>
          <Button
            variant={view === "main" ? "secondary" : "ghost"}
            size="icon"
            class="w-10 h-10 rounded-xl {view === 'main' ? 'bg-primary/15 text-primary hover:bg-primary/25' : 'text-muted-foreground hover:bg-muted'}"
            onclick={onOpenTests}
            aria-label={text.navTests}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true" class="w-5 h-5">
              <path
                d="M7 6h10M7 12h10M7 18h10"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              />
            </svg>
          </Button>
        </Tooltip.Trigger>
        <Tooltip.Content side="right">
          <p>{text.navTests}</p>
        </Tooltip.Content>
      </Tooltip.Root>
    </Tooltip.Provider>
  </div>

  <div class="flex flex-col gap-3 w-full px-2 items-center">
    <Tooltip.Provider>
      <Tooltip.Root>
        <Tooltip.Trigger>
          <Button
            variant={view === "settings" ? "secondary" : "ghost"}
            size="icon"
            class="w-10 h-10 rounded-xl {view === 'settings' ? 'bg-primary/15 text-primary hover:bg-primary/25' : 'text-muted-foreground hover:bg-muted'}"
            onclick={onOpenSettings}
            aria-label={text.navSettings}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true" class="w-5 h-5">
              <path
                d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm7.5-3.5c0-.46-.04-.92-.12-1.35l2.02-1.57-1.9-3.3-2.4.97a7.8 7.8 0 0 0-2.34-1.35l-.37-2.55h-3.8l-.37 2.55a7.8 7.8 0 0 0-2.34 1.35l-2.4-.97-1.9 3.3 2.02 1.57c-.08.43-.12.89-.12 1.35s.04.92.12 1.35l-2.02 1.57 1.9 3.3 2.4-.97c.72.57 1.51 1.04 2.34 1.35l.37 2.55h3.8l.37-2.55c.83-.31 1.62-.78 2.34-1.35l2.4.97 1.9-3.3-2.02-1.57c.08-.43.12-.89.12-1.35Z"
                fill="currentColor"
              />
            </svg>
          </Button>
        </Tooltip.Trigger>
        <Tooltip.Content side="right">
          <p>{text.navSettings}</p>
        </Tooltip.Content>
      </Tooltip.Root>
    </Tooltip.Provider>
  </div>
</aside>


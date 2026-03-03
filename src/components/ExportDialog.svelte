<script lang="ts">
  import { parseDate, type DateValue } from "@internationalized/date";
  import { loadAvailableExportDates } from "../services/tauri";
  import type { Language, Translation } from "../types";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";
  import * as Popover from "$lib/components/ui/popover/index.js";
  import { Calendar } from "$lib/components/ui/calendar/index.js";

  let {
    open,
    exporting,
    text,
    language,
    initialStartDate,
    initialEndDate,
    onClose,
    onConfirm,
  } = $props<{
    open: boolean;
    exporting: boolean;
    text: Translation;
    language: Language;
    initialStartDate: string | null;
    initialEndDate: string | null;
    onClose: () => void;
    onConfirm: (startDate: string, endDate: string) => Promise<void> | void;
  }>();

  const locale = $derived(language === "zh" ? "zh-CN" : "en-US");

  let availableDates = $state<string[]>([]);
  let loadingDates = $state(false);
  let localError = $state<string | null>(null);
  let startValue = $state<DateValue | undefined>(undefined);
  let endValue = $state<DateValue | undefined>(undefined);
  let startPopoverOpen = $state(false);
  let endPopoverOpen = $state(false);
  let wasOpen = $state(false);

  const availableDateSet = $derived(new Set(availableDates));
  const hasAvailableDates = $derived(availableDates.length > 0);
  const startDate = $derived(startValue?.toString() ?? "");
  const endDate = $derived(endValue?.toString() ?? "");

  const disableConfirm = $derived(
    exporting ||
      loadingDates ||
      !hasAvailableDates ||
      startDate.length === 0 ||
      endDate.length === 0,
  );

  $effect(() => {
    if (open && !wasOpen) {
      void loadDates();
    }
    if (!open && wasOpen) {
      resetDialogState();
    }
    wasOpen = open;
  });

  async function loadDates() {
    loadingDates = true;
    localError = null;
    availableDates = [];

    try {
      const dates = await loadAvailableExportDates();
      availableDates = dates;

      if (dates.length === 0) {
        startValue = undefined;
        endValue = undefined;
        return;
      }

      const defaultStart = dates[0];
      const defaultEnd = dates[dates.length - 1];
      const resolvedStart =
        initialStartDate && dates.includes(initialStartDate)
          ? initialStartDate
          : defaultStart;
      const resolvedEnd =
        initialEndDate && dates.includes(initialEndDate)
          ? initialEndDate
          : defaultEnd;

      startValue = parseDate(resolvedStart);
      endValue = parseDate(resolvedEnd);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      localError = `${text.exportDateLoadFailed}: ${message}`;
      startValue = undefined;
      endValue = undefined;
    } finally {
      loadingDates = false;
    }
  }

  function resetDialogState() {
    availableDates = [];
    loadingDates = false;
    localError = null;
    startValue = undefined;
    endValue = undefined;
    startPopoverOpen = false;
    endPopoverOpen = false;
  }

  function isDateDisabled(date: DateValue) {
    return !availableDateSet.has(date.toString());
  }

  async function handleConfirm() {
    if (disableConfirm) return;
    if (startDate > endDate) {
      localError = text.exportInvalidRange;
      return;
    }
    localError = null;
    try {
      await onConfirm(startDate, endDate);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      localError = message;
    }
  }
</script>

<Dialog.Root
  {open}
  onOpenChange={(v) => {
    if (!v) onClose();
  }}
>
  <Dialog.Content class="sm:max-w-[400px]">
    <Dialog.Header>
      <Dialog.Title>{text.exportDialogTitle}</Dialog.Title>
    </Dialog.Header>

    <div class="flex flex-col gap-4 py-2">
      <div class="flex flex-col gap-1.5">
        <span class="text-sm font-medium">{text.exportStartDate}</span>
        <Popover.Root bind:open={startPopoverOpen}>
          <Popover.Trigger
            disabled={exporting || loadingDates || !hasAvailableDates}
            class="border-input bg-background flex h-9 w-full items-center justify-start rounded-md border px-3 py-1 text-sm shadow-xs disabled:opacity-50"
          >
            {startDate || "—"}
          </Popover.Trigger>
          <Popover.Content class="w-auto p-0" align="start">
            <Calendar
              bind:value={startValue}
              {locale}
              {isDateDisabled}
              onValueChange={() => {
                startPopoverOpen = false;
              }}
            />
          </Popover.Content>
        </Popover.Root>
      </div>

      <div class="flex flex-col gap-1.5">
        <span class="text-sm font-medium">{text.exportEndDate}</span>
        <Popover.Root bind:open={endPopoverOpen}>
          <Popover.Trigger
            disabled={exporting || loadingDates || !hasAvailableDates}
            class="border-input bg-background flex h-9 w-full items-center justify-start rounded-md border px-3 py-1 text-sm shadow-xs disabled:opacity-50"
          >
            {endDate || "—"}
          </Popover.Trigger>
          <Popover.Content class="w-auto p-0" align="start">
            <Calendar
              bind:value={endValue}
              {locale}
              {isDateDisabled}
              onValueChange={() => {
                endPopoverOpen = false;
              }}
            />
          </Popover.Content>
        </Popover.Root>
      </div>

      {#if localError}
        <p class="text-destructive text-sm">{localError}</p>
      {:else if !loadingDates && !hasAvailableDates}
        <p class="text-muted-foreground text-sm">{text.exportNoData}</p>
      {/if}
    </div>

    <Dialog.Footer>
      <Button variant="secondary" onclick={onClose} disabled={exporting}>
        {text.exportCancel}
      </Button>
      <Button variant="default" onclick={handleConfirm} disabled={disableConfirm}>
        {exporting ? text.exporting : text.exportConfirm}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

<script lang="ts">
  import { onDestroy, tick } from "svelte";
  import flatpickr from "flatpickr";
  import type { Instance as FlatpickrInstance } from "flatpickr/dist/types/instance";
  import "flatpickr/dist/flatpickr.css";

  import { loadAvailableExportDates } from "../services/tauri";
  import type { Translation } from "../types";
  import { Button } from "$lib/components/ui/button/index.js";

  let {
    open,
    exporting,
    text,
    initialStartDate,
    initialEndDate,
    onClose,
    onConfirm,
  } = $props<{
    open: boolean;
    exporting: boolean;
    text: Translation;
    initialStartDate: string | null;
    initialEndDate: string | null;
    onClose: () => void;
    onConfirm: (startDate: string, endDate: string) => Promise<void> | void;
  }>();

  let startInput = $state<HTMLInputElement | null>(null);
  let endInput = $state<HTMLInputElement | null>(null);
  let startPicker = $state<FlatpickrInstance | null>(null);
  let endPicker = $state<FlatpickrInstance | null>(null);

  let availableDates = $state<string[]>([]);
  let loadingDates = $state(false);
  let localError = $state<string | null>(null);
  let startDate = $state("");
  let endDate = $state("");
  let wasOpen = $state(false);

  const hasAvailableDates = $derived(availableDates.length > 0);
  const disableInputs = $derived(exporting || loadingDates || !hasAvailableDates);
  const disableConfirm = $derived(
    exporting ||
      loadingDates ||
      !hasAvailableDates ||
      startDate.trim().length === 0 ||
      endDate.trim().length === 0,
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

  onDestroy(() => {
    destroyPickers();
  });

  async function loadDates() {
    loadingDates = true;
    localError = null;
    availableDates = [];

    try {
      const dates = await loadAvailableExportDates();
      availableDates = dates;

      if (dates.length === 0) {
        startDate = "";
        endDate = "";
        destroyPickers();
        return;
      }

      const defaultStart = dates[0];
      const defaultEnd = dates[dates.length - 1];
      startDate = initialStartDate && dates.includes(initialStartDate) ? initialStartDate : defaultStart;
      endDate = initialEndDate && dates.includes(initialEndDate) ? initialEndDate : defaultEnd;

      await tick();
      initPickers();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      localError = `${text.exportDateLoadFailed}: ${message}`;
      startDate = "";
      endDate = "";
      destroyPickers();
    } finally {
      loadingDates = false;
    }
  }

  function initPickers() {
    if (!startInput || !endInput) {
      return;
    }

    destroyPickers();

    startPicker = flatpickr(startInput, {
      dateFormat: "Y-m-d",
      allowInput: false,
      enable: availableDates,
      defaultDate: startDate,
      onChange: (_selectedDates, dateStr) => {
        startDate = dateStr;
      },
    });

    endPicker = flatpickr(endInput, {
      dateFormat: "Y-m-d",
      allowInput: false,
      enable: availableDates,
      defaultDate: endDate,
      onChange: (_selectedDates, dateStr) => {
        endDate = dateStr;
      },
    });
  }

  function destroyPickers() {
    if (startPicker) {
      startPicker.destroy();
      startPicker = null;
    }
    if (endPicker) {
      endPicker.destroy();
      endPicker = null;
    }
  }

  function resetDialogState() {
    destroyPickers();
    availableDates = [];
    loadingDates = false;
    localError = null;
    startDate = "";
    endDate = "";
  }

  async function handleConfirm() {
    if (disableConfirm) {
      return;
    }
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

{#if open}
  <div class="overlay" role="presentation">
    <div class="dialog" role="dialog" aria-modal="true" aria-label={text.exportDialogTitle}>
      <h3>{text.exportDialogTitle}</h3>

      <label>
        {text.exportStartDate}
        <input bind:this={startInput} type="text" value={startDate} disabled={disableInputs} />
      </label>

      <label>
        {text.exportEndDate}
        <input bind:this={endInput} type="text" value={endDate} disabled={disableInputs} />
      </label>

      {#if localError}
        <p class="dialog-error">{localError}</p>
      {:else if !loadingDates && !hasAvailableDates}
        <p class="dialog-note">{text.exportNoData}</p>
      {/if}

      <div class="actions">
        <Button variant="secondary" onclick={onClose} disabled={exporting}>
          {text.exportCancel}
        </Button>
        <Button variant="default" onclick={handleConfirm} disabled={disableConfirm}>
          {exporting ? text.exporting : text.exportConfirm}
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    width: min(420px, calc(100vw - 2rem));
    background: #fff;
    border-radius: 10px;
    padding: 1rem;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .dialog label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.92rem;
  }

  .dialog input[type="text"] {
    padding: 0.45rem 0.55rem;
  }

  .dialog-error {
    margin: 0;
    color: #b42318;
    font-size: 0.9rem;
  }

  .dialog-note {
    margin: 0;
    color: #475569;
    font-size: 0.9rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .actions :global(button) {
    padding: 8px 16px;
    font-size: 14px;
  }
</style>

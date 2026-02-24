<script lang="ts">
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
    onSave,
  } = $props<{
    text: Translation;
    settingsDraft: SettingsDraft | null;
    settingsSaving: boolean;
    settingsSaved: boolean;
    settingsError: string | null;
    onSave: () => void;
  }>();
</script>

<div class="settings-card">
  {#if settingsDraft === null}
    <p class="settings-message">{text.settingsLoading}</p>
  {:else}
    <div class="field">
      <span class="field-label">{text.fieldMode}</span>
      <div class="segmented" role="group" aria-label={text.fieldMode}>
        <button
          class="segment"
          data-active={settingsDraft.mode === "network"}
          onclick={() => {
            if (settingsDraft) settingsDraft.mode = "network";
          }}
        >
          {text.fieldNetwork}
        </button>
        <button
          class="segment"
          data-active={settingsDraft.mode === "serial"}
          onclick={() => {
            if (settingsDraft) settingsDraft.mode = "serial";
          }}
        >
          {text.fieldSerial}
        </button>
      </div>
    </div>

    {#if settingsDraft.mode === "network"}
      <div class="field-grid">
        <div class="field">
          <label for="field-ip">{text.fieldIp}</label>
          <input
            id="field-ip"
            type="text"
            value={settingsDraft.ip_address}
            oninput={(event) => {
              if (settingsDraft)
                settingsDraft.ip_address = event.currentTarget.value;
            }}
          />
        </div>
        <div class="field">
          <label for="field-port">{text.fieldPort}</label>
          <input
            id="field-port"
            type="text"
            value={settingsDraft.port}
            oninput={(event) => {
              if (settingsDraft) settingsDraft.port = event.currentTarget.value;
            }}
          />
        </div>
      </div>
    {:else}
      <div class="field">
        <label for="field-serial-port">{text.fieldSerialPort}</label>
        <input
          id="field-serial-port"
          type="number"
          min="1"
          value={settingsDraft.port_number}
          oninput={(event) => {
            if (!settingsDraft) return;
            const value = Number(event.currentTarget.value);
            settingsDraft.port_number = Number.isNaN(value) ? 1 : value;
          }}
        />
      </div>
    {/if}

    <div class="field">
      <label for="field-timeout">{text.fieldTimeout}</label>
      <input
        id="field-timeout"
        type="number"
        min="0"
        value={settingsDraft.read_timeout_ms}
        oninput={(event) => {
          if (!settingsDraft) return;
          const value = Number(event.currentTarget.value);
          settingsDraft.read_timeout_ms = Number.isNaN(value) ? 0 : value;
        }}
      />
    </div>

    <div class="settings-actions">
      <button class="primary" onclick={onSave} disabled={settingsSaving}>
        {settingsSaving ? text.settingsSaving : text.settingsSave}
      </button>
      {#if settingsSaved}
        <span class="settings-success">{text.settingsSaved}</span>
      {/if}
      {#if settingsError}
        <span class="settings-error">
          {text.settingsError}: {settingsError}
        </span>
      {/if}
    </div>
  {/if}
</div>

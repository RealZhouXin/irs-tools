<script lang="ts">
  import type { Translation } from "../types";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as Select from "$lib/components/ui/select/index.js";

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

<Card.Root>
  <Card.Header>
    <Card.Title>连接与系统设置</Card.Title>
    <Card.Description>修改将会在下次启动或测试时生效</Card.Description>
  </Card.Header>
  <Card.Content class="space-y-6">
    {#if settingsDraft === null}
      <p class="text-sm text-muted-foreground">{text.settingsLoading}</p>
    {:else}
      <div class="space-y-3">
        <Label>{text.fieldMode}</Label>
        <div class="inline-flex h-9 items-center justify-center rounded-lg bg-muted p-1 text-muted-foreground" role="group" aria-label={text.fieldMode}>
          <button
            class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[active=true]:bg-background data-[active=true]:text-foreground data-[active=true]:shadow"
            data-active={settingsDraft.mode === "network"}
            onclick={() => {
              if (settingsDraft) settingsDraft.mode = "network";
            }}
          >
            {text.fieldNetwork}
          </button>
          <button
            class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[active=true]:bg-background data-[active=true]:text-foreground data-[active=true]:shadow"
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
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-2">
            <Label for="field-ip">{text.fieldIp}</Label>
            <Input
              id="field-ip"
              type="text"
              value={settingsDraft.ip_address}
              oninput={(event) => {
                if (settingsDraft)
                  settingsDraft.ip_address = event.currentTarget.value;
              }}
            />
          </div>
          <div class="space-y-2">
            <Label for="field-port">{text.fieldPort}</Label>
            <Input
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
        <div class="space-y-2">
          <Label for="field-serial-port">{text.fieldSerialPort}</Label>
          <Input
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

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div class="space-y-2">
          <Label for="field-timeout">{text.fieldTimeout}</Label>
          <Input
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

        <div class="space-y-2">
          <Label for="field-log-level">{text.fieldLogLevel}</Label>
          <select
            id="field-log-level"
            class="flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1"
            value={settingsDraft.log_level}
            onchange={(event) => {
              if (!settingsDraft) return;
              settingsDraft.log_level = event.currentTarget
                .value as SettingsDraft["log_level"];
            }}
          >
            <option value="error">{text.logLevelError}</option>
            <option value="warn">{text.logLevelWarn}</option>
            <option value="info">{text.logLevelInfo}</option>
            <option value="debug">{text.logLevelDebug}</option>
            <option value="trace">{text.logLevelTrace}</option>
          </select>
        </div>
      </div>
    {/if}
  </Card.Content>
  <Card.Footer class="flex flex-col items-start gap-4 sm:flex-row sm:items-center">
    {#if settingsDraft !== null}
       <Button variant="default" onclick={onSave} disabled={settingsSaving}>
          {settingsSaving ? text.settingsSaving : text.settingsSave}
       </Button>
       {#if settingsSaved}
          <span class="text-sm font-medium text-green-600">{text.settingsSaved}</span>
       {/if}
       {#if settingsError}
          <span class="text-sm font-medium text-destructive">
             {text.settingsError}: {settingsError}
          </span>
       {/if}
    {/if}
  </Card.Footer>
</Card.Root>

<script lang="ts">
  import * as Select from "$lib/components/ui/select/index.js";
  import type { Translation } from "../types";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";

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

<div class="bg-white border border-zinc-200/60 shadow-sm rounded-xl overflow-hidden">
  <div class="p-6">
    {#if settingsDraft === null}
      <div class="py-10 flex justify-center items-center">
        <div class="h-5 w-5 rounded-full border-2 border-zinc-300 border-t-zinc-900 animate-spin"></div>
        <span class="ml-3 text-sm text-zinc-500">{text.settingsLoading}</span>
      </div>
    {:else}
      <div class="space-y-6">
        <!-- 通信模式 -->
        <div class="space-y-3">
          <Label class="text-zinc-600 text-[13px] uppercase tracking-wider font-semibold">{text.fieldMode}</Label>
          <div class="inline-flex bg-zinc-100/80 p-1 rounded-lg border border-zinc-200/50" role="group">
            <button
              class="px-4 py-1.5 text-sm font-medium rounded-md transition-all {settingsDraft.mode === 'network' ? 'bg-white text-zinc-900 shadow-sm ring-1 ring-zinc-200/50' : 'text-zinc-500 hover:text-zinc-900 hover:bg-zinc-200/30'}"
              onclick={() => {
                if (settingsDraft) settingsDraft.mode = "network";
              }}
            >
              {text.fieldNetwork}
            </button>
            <button
              class="px-4 py-1.5 text-sm font-medium rounded-md transition-all {settingsDraft.mode === 'serial' ? 'bg-white text-zinc-900 shadow-sm ring-1 ring-zinc-200/50' : 'text-zinc-500 hover:text-zinc-900 hover:bg-zinc-200/30'}"
              onclick={() => {
                if (settingsDraft) settingsDraft.mode = "serial";
              }}
            >
              {text.fieldSerial}
            </button>
          </div>
        </div>

        <div class="h-px bg-zinc-100"></div>

        <!-- 连接参数 -->
        {#if settingsDraft.mode === "network"}
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
            <div class="space-y-2">
              <Label for="field-ip" class="text-zinc-700">{text.fieldIp}</Label>
              <Input
                id="field-ip"
                type="text"
                class="bg-zinc-50/50 border-zinc-200 focus:bg-white focus:ring-zinc-900 transition-colors"
                value={settingsDraft.ip_address}
                oninput={(event) => {
                  if (settingsDraft)
                    settingsDraft.ip_address = event.currentTarget.value;
                }}
              />
            </div>
            <div class="space-y-2">
              <Label for="field-port" class="text-zinc-700">{text.fieldPort}</Label>
              <Input
                id="field-port"
                type="text"
                class="bg-zinc-50/50 border-zinc-200 focus:bg-white focus:ring-zinc-900 transition-colors"
                value={settingsDraft.port}
                oninput={(event) => {
                  if (settingsDraft) settingsDraft.port = event.currentTarget.value;
                }}
              />
            </div>
          </div>
        {:else}
          <div class="space-y-2">
            <Label for="field-serial-port" class="text-zinc-700">{text.fieldSerialPort}</Label>
            <Input
              id="field-serial-port"
              type="number"
              min="1"
              class="bg-zinc-50/50 border-zinc-200 focus:bg-white focus:ring-zinc-900 transition-colors max-w-xs"
              value={settingsDraft.port_number}
              oninput={(event) => {
                if (!settingsDraft) return;
                const value = Number(event.currentTarget.value);
                settingsDraft.port_number = Number.isNaN(value) ? 1 : value;
              }}
            />
          </div>
        {/if}

        <!-- 其他参数 -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-6 pt-2">
          <div class="space-y-2">
            <Label for="field-timeout" class="text-zinc-700">{text.fieldTimeout}</Label>
            <div class="relative">
              <Input
                id="field-timeout"
                type="number"
                min="0"
                class="bg-zinc-50/50 border-zinc-200 focus:bg-white focus:ring-zinc-900 transition-colors pr-10"
                value={settingsDraft.read_timeout_ms}
                oninput={(event) => {
                  if (!settingsDraft) return;
                  const value = Number(event.currentTarget.value);
                  settingsDraft.read_timeout_ms = Number.isNaN(value) ? 0 : value;
                }}
              />
              <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[13px] text-zinc-400 font-medium">ms</span>
            </div>
          </div>

          <div class="space-y-2">
            <Label for="field-log-level" class="text-zinc-700">{text.fieldLogLevel}</Label>
            <Select.Root
              type="single"
              value={settingsDraft.log_level}
              onValueChange={(v) => { if (settingsDraft) settingsDraft.log_level = v as any; }}
            >
              <Select.Trigger
                id="field-log-level"
                class="flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-zinc-200 bg-zinc-50/50 px-3 py-2 text-sm shadow-sm transition-colors focus:outline-none focus:ring-1 focus:ring-zinc-900 focus:bg-white cursor-pointer"
              >
                {settingsDraft.log_level === "error" ? text.logLevelError :
                 settingsDraft.log_level === "warn" ? text.logLevelWarn :
                 settingsDraft.log_level === "info" ? text.logLevelInfo :
                 settingsDraft.log_level === "debug" ? text.logLevelDebug :
                 text.logLevelTrace}
              </Select.Trigger>
              <Select.Content>
                <Select.Item value="error" label={text.logLevelError} />
                <Select.Item value="warn" label={text.logLevelWarn} />
                <Select.Item value="info" label={text.logLevelInfo} />
                <Select.Item value="debug" label={text.logLevelDebug} />
                <Select.Item value="trace" label={text.logLevelTrace} />
              </Select.Content>
            </Select.Root>
          </div>
        </div>
      </div>
    {/if}
  </div>

  {#if settingsDraft !== null}
    <div class="px-6 py-4 bg-zinc-50 border-t border-zinc-200/60 flex items-center justify-between">
      <div class="flex items-center gap-3">
        {#if settingsSaved}
          <div class="flex items-center gap-1.5 text-emerald-600 bg-emerald-50 px-2.5 py-1 rounded-md text-sm font-medium border border-emerald-100 animate-in fade-in slide-in-from-left-2">
            <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
            {text.settingsSaved}
          </div>
        {/if}
        {#if settingsError}
          <div class="flex items-center gap-1.5 text-rose-600 bg-rose-50 px-2.5 py-1 rounded-md text-sm font-medium border border-rose-100 flex-1 max-w-sm truncate animate-in fade-in slide-in-from-left-2 cursor-help" title={settingsError}>
            <svg class="w-4 h-4 shrink-0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>
            <span class="truncate">{text.settingsError}: {settingsError}</span>
          </div>
        {/if}
      </div>
      
      <Button 
        class="bg-zinc-900 hover:bg-zinc-800 text-white min-w-[100px] shadow-sm transition-all active:scale-[0.98]" 
        onclick={onSave} 
        disabled={settingsSaving}
      >
        {#if settingsSaving}
          <div class="h-4 w-4 rounded-full border-2 border-white/30 border-t-white animate-spin mr-2"></div>
          {text.settingsSaving}
        {:else}
          {text.settingsSave}
        {/if}
      </Button>
    </div>
  {/if}
</div>


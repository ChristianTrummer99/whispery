<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import {
    loadSettings,
    saveSettings,
    getLlmUrl,
    getLlmModel,
    getLlmApiKey,
    type Settings,
  } from "$lib/stores";
  import ApiSettings from "$lib/components/ApiSettings.svelte";
  import PromptManager from "$lib/components/PromptManager.svelte";
  import AudioSettings from "$lib/components/AudioSettings.svelte";

  let settings = $state<Settings | null>(null);
  let activeTab = $state<"prompts" | "api" | "audio">("prompts");
  let status = $state<"idle" | "listening" | "processing" | "success" | "error">("idle");
  let statusMessage = $state("");
  let lastResult = $state("");
  let isRecording = $state(false);
  let testResult = $state("");
  let unlistenStatus: (() => void) | null = null;
  let registeredKey: string | null = null;

  onMount(async () => {
    settings = await loadSettings();

    const unlisten = await listen<{ state: string; message?: string }>(
      "recording-status",
      (event) => {
        status = event.payload.state as typeof status;
        statusMessage = event.payload.message ?? "";
        if (status === "success") {
          lastResult = statusMessage;
        }
      }
    );
    unlistenStatus = unlisten;

    await registerShortcut();
  });

  onDestroy(() => {
    unlistenStatus?.();
    unregisterAll();
  });

  async function startCapture() {
    isRecording = true;
    try {
      await invoke("start_recording", {
        deviceName: settings!.selectedDevice || null,
      });
    } catch (e) {
      console.error("Failed to start recording:", e);
      isRecording = false;
    }
  }

  async function stopAndProcess() {
    isRecording = false;
    try {
      const activePrompt = settings!.prompts.find(
        (p) => p.id === settings!.activePromptId
      );
      const result = await invoke<string>("stop_recording_and_process", {
        openaiApiKey: settings!.openaiApiKey,
        llmApiKey: getLlmApiKey(settings!),
        llmApiUrl: getLlmUrl(settings!),
        llmModel: getLlmModel(settings!),
        promptTemplate: activePrompt?.template ?? "",
        skipTransform: !activePrompt?.template,
        whisperLanguage: settings!.whisperLanguage || "en",
      });

      if (result) {
        await invoke("test_paste_combined", { text: result });
      }
    } catch (e) {
      console.error("Processing failed:", e);
    }
  }

  async function registerShortcut() {
    if (!settings) return;

    try {
      if (registeredKey) {
        await unregisterAll();
        registeredKey = null;
      }

      const key = settings.pttKey;
      const mode = settings.pttMode;

      await register(key, async (event) => {
        if (mode === "hold") {
          if (event.state === "Pressed" && !isRecording) {
            await startCapture();
          } else if (event.state === "Released" && isRecording) {
            await stopAndProcess();
          }
        } else {
          if (event.state === "Pressed") {
            if (!isRecording) {
              await startCapture();
            } else {
              await stopAndProcess();
            }
          }
        }
      });
      registeredKey = key;
    } catch (e) {
      console.error("Failed to register shortcut:", e);
    }
  }

  async function handleSave() {
    if (!settings) return;
    await saveSettings(settings);
    await registerShortcut();
  }

  const tabs = [
    { id: "prompts" as const, label: "Prompts", icon: "M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" },
    { id: "api" as const, label: "API Keys", icon: "M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" },
    { id: "audio" as const, label: "Audio", icon: "M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" },
  ];

  const statusColors: Record<string, string> = {
    idle: "bg-surface-lighter",
    listening: "bg-accent/20 text-accent",
    processing: "bg-warning/20 text-warning",
    success: "bg-success/20 text-success",
    error: "bg-danger/20 text-danger",
  };
</script>

{#if settings}
  <div class="min-h-screen bg-surface text-text">
    <!-- Header -->
    <header class="border-b border-surface-lighter bg-surface/80 backdrop-blur-sm sticky top-0 z-10">
      <div class="max-w-4xl mx-auto px-6 py-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-primary to-accent flex items-center justify-center">
            <svg class="w-4.5 h-4.5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
              <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
            </svg>
          </div>
          <h1 class="text-lg font-bold tracking-tight">Whispery</h1>
        </div>

        <!-- Status Badge -->
        <div class="flex items-center gap-3">
          <div class="px-3 py-1.5 rounded-full text-xs font-medium capitalize {statusColors[status] || statusColors.idle}">
            {#if status === "idle"}
              Ready &middot; {settings.pttMode === "hold" ? "Hold" : "Press"} <kbd class="px-1.5 py-0.5 bg-surface rounded text-[10px] font-mono">{settings.pttKey}</kbd>
            {:else}
              {status}
            {/if}
          </div>
        </div>
      </div>
    </header>

    <div class="max-w-4xl mx-auto px-6 py-6">
      <!-- Last Result -->
      {#if lastResult}
        <div class="mb-6 p-4 bg-surface-light rounded-xl border border-surface-lighter">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-text-muted uppercase tracking-wider">Last Result</span>
            <button
              onclick={() => writeText(lastResult)}
              class="text-xs text-primary hover:text-primary-light transition-colors font-medium"
            >
              Copy Again
            </button>
          </div>
          <p class="text-sm text-text leading-relaxed">{lastResult}</p>
        </div>
      {/if}

      <!-- Tabs -->
      <nav class="flex gap-1 mb-6 p-1 bg-surface-light rounded-xl">
        {#each tabs as tab}
          <button
            onclick={() => (activeTab = tab.id)}
            class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === tab.id
              ? 'bg-surface text-text shadow-sm'
              : 'text-text-muted hover:text-text'}"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={tab.icon} />
            </svg>
            {tab.label}
          </button>
        {/each}
      </nav>

      <!-- Tab Content -->
      <div class="bg-surface-light/50 rounded-2xl border border-surface-lighter p-6">
        {#if activeTab === "prompts"}
          <PromptManager bind:settings onSave={handleSave} />
        {:else if activeTab === "api"}
          <ApiSettings bind:settings onSave={handleSave} />
        {:else if activeTab === "audio"}
          <AudioSettings bind:settings onSave={handleSave} />
        {/if}
      </div>

      <!-- Active Prompt Indicator -->
      <div class="mt-6 text-center">
        <span class="text-xs text-text-muted">
          Active: <span class="text-primary font-medium">
            {settings.prompts.find((p) => p.id === settings.activePromptId)?.name ?? "None"}
          </span>
        </span>
      </div>

      <!-- Debug: Paste Test Panel -->
      <div class="mt-6 p-4 bg-surface-light rounded-xl border border-surface-lighter">
        <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider mb-3">Paste Debug Tests</h3>
        <p class="text-xs text-text-muted mb-3">Click a button, then click into the textarea within 3 seconds. The paste will fire after the countdown.</p>
        <textarea class="w-full bg-surface border border-surface-lighter rounded-lg px-3 py-2 text-sm text-text mb-3 h-20" placeholder="Text will appear here after the 3s countdown..."></textarea>
        <div class="flex gap-2 flex-wrap">
          <button
            onclick={() => {
              testResult = "⏳ 3s... click into the textarea now!";
              setTimeout(async () => {
                try {
                  const r = await invoke("test_clipboard_only", { text: "hello clipboard" });
                  testResult = "Clipboard: " + r;
                } catch (e) { testResult = "Clipboard ERROR: " + e; }
              }, 3000);
            }}
            class="px-3 py-1.5 bg-primary/10 text-primary rounded-lg text-xs font-medium hover:bg-primary/20 transition-colors"
          >1. Test Clipboard Only</button>
          <button
            onclick={() => {
              testResult = "⏳ 3s... click into the textarea now!";
              setTimeout(async () => {
                try {
                  const r = await invoke("test_enigo_only");
                  testResult = "Enigo: " + r;
                } catch (e) { testResult = "Enigo ERROR: " + e; }
              }, 3000);
            }}
            class="px-3 py-1.5 bg-primary/10 text-primary rounded-lg text-xs font-medium hover:bg-primary/20 transition-colors"
          >2. Test Enigo Only</button>
          <button
            onclick={() => {
              testResult = "⏳ 3s... click into the textarea now!";
              setTimeout(async () => {
                try {
                  const r = await invoke("test_paste_combined", { text: "hello paste!" });
                  testResult = "Combined: " + r;
                } catch (e) { testResult = "Combined ERROR: " + e; }
              }, 3000);
            }}
            class="px-3 py-1.5 bg-primary/10 text-primary rounded-lg text-xs font-medium hover:bg-primary/20 transition-colors"
          >3. Test Full Paste</button>
        </div>
        {#if testResult}
          <p class="mt-2 text-xs font-mono {testResult.includes('ERROR') ? 'text-danger' : testResult.includes('⏳') ? 'text-warning' : 'text-success'}">{testResult}</p>
        {/if}
      </div>
    </div>
  </div>
{:else}
  <div class="min-h-screen bg-surface flex items-center justify-center">
    <div class="text-text-muted animate-pulse">Loading...</div>
  </div>
{/if}

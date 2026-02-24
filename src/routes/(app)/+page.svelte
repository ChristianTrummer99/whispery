<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { register, unregister, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
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
  import GlossarySettings from "$lib/components/GlossarySettings.svelte";

  let settings = $state<Settings | null>(null);
  let activeTab = $state<"prompts" | "api" | "audio" | "glossary">("prompts");
  let status = $state<"idle" | "listening" | "processing" | "success" | "error">("idle");
  let statusMessage = $state("");
  let lastResult = $state("");
  let isRecording = $state(false);
  let isProcessing = $state(false);
  let unlistenStatus: (() => void) | null = null;
  let registeredKey: string | null = null;
  let escapeRegistered = false;

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

  async function registerEscape() {
    if (escapeRegistered) return;
    try {
      await register("Escape", async (event) => {
        if (event.state === "Pressed" && isRecording && !isProcessing) {
          await cancelCapture();
        }
      });
      escapeRegistered = true;
    } catch (e) {
      console.error("Failed to register Escape:", e);
    }
  }

  async function unregisterEscape() {
    if (!escapeRegistered) return;
    try {
      await unregister("Escape");
      escapeRegistered = false;
    } catch (e) {
      console.error("Failed to unregister Escape:", e);
    }
  }

  async function startCapture() {
    isRecording = true;
    try {
      await invoke("start_recording", {
        deviceName: settings!.selectedDevice || null,
      });
      if (settings!.pttMode === "toggle") {
        await registerEscape();
      }
    } catch (e) {
      console.error("Failed to start recording:", e);
      isRecording = false;
    }
  }

  async function cancelCapture() {
    isRecording = false;
    await unregisterEscape();
    try {
      await invoke("cancel_recording");
    } catch (e) {
      console.error("Failed to cancel recording:", e);
    }
  }

  async function stopAndProcess() {
    isRecording = false;
    isProcessing = true;
    await unregisterEscape();
    try {
      const activePrompt = settings!.prompts.find(
        (p) => p.id === settings!.activePromptId
      );
      const glossaryPrompt = settings!.glossary.length > 0
        ? settings!.glossary.join(", ")
        : null;

      const result = await invoke<string>("stop_recording_and_process", {
        openaiApiKey: settings!.openaiApiKey,
        llmApiKey: getLlmApiKey(settings!),
        llmApiUrl: getLlmUrl(settings!),
        llmModel: getLlmModel(settings!),
        promptTemplate: activePrompt?.template ?? "",
        skipTransform: !activePrompt?.template,
        whisperLanguage: settings!.whisperLanguage || "en",
        glossaryPrompt,
        saveDebugAudio: settings!.saveDebugAudio || false,
      });

      if (result) {
        await invoke("paste_to_input", { text: result });
      }
    } catch (e) {
      console.error("Processing failed:", e);
    } finally {
      isProcessing = false;
    }
  }

  async function registerShortcut() {
    if (!settings) return;

    try {
      if (registeredKey) {
        await unregisterAll();
        registeredKey = null;
        escapeRegistered = false;
      }

      const key = settings.pttKey;
      const mode = settings.pttMode;

      await register(key, async (event) => {
        if (mode === "hold") {
          if (event.state === "Pressed" && !isRecording && !isProcessing) {
            await startCapture();
          } else if (event.state === "Released" && isRecording) {
            await stopAndProcess();
          }
        } else {
          if (event.state === "Pressed") {
            if (isProcessing) return;
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
    { id: "glossary" as const, label: "Glossary", icon: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" },
    { id: "api" as const, label: "API Keys", icon: "M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" },
    { id: "audio" as const, label: "Audio", icon: "M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z" },
  ];

  const statusColors: Record<string, string> = {
    idle: "bg-surface-lighter/60 text-text-muted",
    listening: "bg-success/10 text-success",
    processing: "bg-warning/10 text-warning",
    success: "bg-success/10 text-success",
    error: "bg-danger/10 text-danger",
  };
</script>

{#if settings}
  <div class="min-h-screen bg-surface-light text-text">
    <header class="border-b border-surface-lighter bg-white sticky top-0 z-10">
      <div class="max-w-4xl mx-auto px-6 py-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
            <svg class="w-4.5 h-4.5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
              <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
            </svg>
          </div>
          <h1 class="text-lg font-bold tracking-tight">Whispery</h1>
        </div>

        <div class="flex items-center gap-3">
          <div class="px-3 py-1.5 rounded-full text-xs font-medium capitalize {statusColors[status] || statusColors.idle}">
            {#if status === "idle"}
              Ready &middot; {settings.pttMode === "hold" ? "Hold" : "Press"} <kbd class="px-1.5 py-0.5 bg-surface-lighter rounded text-[10px] font-mono">{settings.pttKey}</kbd>
            {:else}
              {status}
            {/if}
          </div>
        </div>
      </div>
    </header>

    <div class="max-w-4xl mx-auto px-6 py-6">
      {#if lastResult}
        <div class="mb-6 p-4 bg-white rounded-xl border border-surface-lighter shadow-sm">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-semibold text-text-muted uppercase tracking-wider">Last Result</span>
            <button
              onclick={() => writeText(lastResult)}
              class="text-xs text-text-muted hover:text-text transition-colors font-medium"
            >
              Copy Again
            </button>
          </div>
          <p class="text-sm text-text leading-relaxed">{lastResult}</p>
        </div>
      {/if}

      <nav class="flex gap-1 mb-6 p-1 bg-surface-lighter/50 rounded-xl">
        {#each tabs as tab}
          <button
            onclick={() => (activeTab = tab.id)}
            class="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === tab.id
              ? 'bg-white text-text shadow-sm'
              : 'text-text-muted hover:text-text'}"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={tab.icon} />
            </svg>
            {tab.label}
          </button>
        {/each}
      </nav>

      <div class="bg-white rounded-2xl border border-surface-lighter p-6 shadow-sm">
        {#if activeTab === "prompts"}
          <PromptManager bind:settings onSave={handleSave} />
        {:else if activeTab === "glossary"}
          <GlossarySettings bind:settings onSave={handleSave} />
        {:else if activeTab === "api"}
          <ApiSettings bind:settings onSave={handleSave} />
        {:else if activeTab === "audio"}
          <AudioSettings bind:settings onSave={handleSave} />
        {/if}
      </div>

      <div class="mt-6 text-center">
        <span class="text-xs text-text-muted">
          Active: <span class="text-text font-medium">
            {settings?.prompts.find((p) => p.id === settings?.activePromptId)?.name ?? "None"}
          </span>
        </span>
      </div>
    </div>
  </div>
{:else}
  <div class="min-h-screen bg-surface-light flex items-center justify-center">
    <div class="text-text-muted animate-pulse">Loading...</div>
  </div>
{/if}

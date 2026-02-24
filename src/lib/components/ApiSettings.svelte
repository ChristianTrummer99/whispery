<script lang="ts">
  import type { Settings } from "$lib/stores";

  let { settings = $bindable(), onSave }: { settings: Settings; onSave: () => void } = $props();
  let showOpenaiKey = $state(false);
  let showLlmKey = $state(false);

  const providers = [
    { id: "openai" as const, name: "OpenAI", url: "https://api.openai.com/v1/chat/completions", models: ["gpt-4o-mini", "gpt-4o", "gpt-4-turbo"] },
    { id: "anthropic" as const, name: "Anthropic", url: "https://api.anthropic.com/v1/messages", models: ["claude-sonnet-4-20250514", "claude-3-5-haiku-20241022"] },
    { id: "custom" as const, name: "Custom Endpoint", url: "", models: [] },
  ];

  function onProviderChange() {
    const p = providers.find((p) => p.id === settings.llmProvider);
    if (p && p.id !== "custom") {
      settings.llmApiUrl = p.url;
      settings.llmModel = p.models[0] ?? "";
    }
    onSave();
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-4">
      Speech-to-Text (Whisper)
    </h3>
    <label for="openai-key" class="block mb-1 text-sm text-text">OpenAI API Key</label>
    <div class="relative">
      <input
        id="openai-key"
        type={showOpenaiKey ? "text" : "password"}
        bind:value={settings.openaiApiKey}
        onchange={onSave}
        placeholder="sk-..."
        class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text placeholder-text-muted/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors pr-10"
      />
      <button
        type="button"
        onclick={() => (showOpenaiKey = !showOpenaiKey)}
        class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text transition-colors"
      >
        {showOpenaiKey ? "Hide" : "Show"}
      </button>
    </div>
    <p class="mt-1.5 text-xs text-text-muted">Used for Whisper transcription. Get one at platform.openai.com</p>
  </div>

  <hr class="border-surface-lighter" />

  <div>
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-4">
      Text Transformation (LLM)
    </h3>

    <label for="llm-provider" class="block mb-1 text-sm text-text">Provider</label>
    <select
      id="llm-provider"
      bind:value={settings.llmProvider}
      onchange={onProviderChange}
      class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
    >
      {#each providers as p}
        <option value={p.id}>{p.name}</option>
      {/each}
    </select>

    <div class="mt-4">
      <label for="llm-key" class="block mb-1 text-sm text-text">LLM API Key</label>
      <div class="relative">
        <input
          id="llm-key"
          type={showLlmKey ? "text" : "password"}
          bind:value={settings.llmApiKey}
          onchange={onSave}
          placeholder="API key..."
          class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text placeholder-text-muted/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors pr-10"
        />
        <button
          type="button"
          onclick={() => (showLlmKey = !showLlmKey)}
          class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text transition-colors"
        >
          {showLlmKey ? "Hide" : "Show"}
        </button>
      </div>
      <p class="mt-1 text-xs text-text-muted">Leave blank to reuse OpenAI key</p>
    </div>

    {#if settings.llmProvider === "custom"}
      <div class="mt-4">
        <label for="llm-url" class="block mb-1 text-sm text-text">API URL</label>
        <input
          id="llm-url"
          type="text"
          bind:value={settings.llmApiUrl}
          onchange={onSave}
          placeholder="https://..."
          class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text placeholder-text-muted/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
        />
      </div>
    {/if}

    <div class="mt-4">
      <label for="llm-model" class="block mb-1 text-sm text-text">Model</label>
      {#if settings.llmProvider !== "custom"}
        {@const currentProvider = providers.find((p) => p.id === settings.llmProvider)}
        <select
          id="llm-model"
          bind:value={settings.llmModel}
          onchange={onSave}
          class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
        >
          {#each currentProvider?.models ?? [] as model}
            <option value={model}>{model}</option>
          {/each}
        </select>
      {:else}
        <input
          id="llm-model"
          type="text"
          bind:value={settings.llmModel}
          onchange={onSave}
          placeholder="model-name"
          class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text placeholder-text-muted/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
        />
      {/if}
    </div>
  </div>
</div>

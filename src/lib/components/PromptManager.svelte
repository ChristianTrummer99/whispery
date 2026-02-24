<script lang="ts">
  import type { Settings, Prompt } from "$lib/stores";

  let { settings = $bindable(), onSave }: { settings: Settings; onSave: () => void } = $props();
  let editingId = $state<string | null>(null);
  let editName = $state("");
  let editTemplate = $state("");

  function selectPrompt(id: string) {
    settings.activePromptId = id;
    onSave();
  }

  function startEdit(prompt: Prompt) {
    editingId = prompt.id;
    editName = prompt.name;
    editTemplate = prompt.template;
  }

  function saveEdit() {
    if (!editingId) return;
    const idx = settings.prompts.findIndex((p) => p.id === editingId);
    if (idx >= 0) {
      settings.prompts[idx] = { ...settings.prompts[idx], name: editName, template: editTemplate };
      settings.prompts = [...settings.prompts];
    }
    editingId = null;
    onSave();
  }

  function cancelEdit() {
    editingId = null;
  }

  function addPrompt() {
    const id = `custom-${Date.now()}`;
    settings.prompts = [
      ...settings.prompts,
      { id, name: "New Prompt", template: "Transform the following text:\n\n{text}" },
    ];
    settings.activePromptId = id;
    startEdit(settings.prompts[settings.prompts.length - 1]);
    onSave();
  }

  function deletePrompt(id: string) {
    if (settings.prompts.length <= 1) return;
    settings.prompts = settings.prompts.filter((p) => p.id !== id);
    if (settings.activePromptId === id) {
      settings.activePromptId = settings.prompts[0]?.id ?? "";
    }
    if (editingId === id) editingId = null;
    onSave();
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between mb-2">
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider">
      Transformation Prompts
    </h3>
    <button
      onclick={addPrompt}
      class="flex items-center gap-1.5 px-3 py-1.5 bg-primary/10 text-primary hover:bg-primary/20 rounded-lg text-sm font-medium transition-colors"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Add Prompt
    </button>
  </div>

  <div class="space-y-2">
    {#each settings.prompts as prompt (prompt.id)}
      {#if editingId === prompt.id}
        <div class="bg-surface-light border border-primary/30 rounded-xl p-4 space-y-3">
          <input
            bind:value={editName}
            placeholder="Prompt name"
            class="w-full bg-surface border border-surface-lighter rounded-lg px-3 py-2 text-text text-sm focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
          />
          <textarea
            bind:value={editTemplate}
            placeholder="Use {'{'}text{'}'} as placeholder for the transcribed text"
            rows="4"
            class="w-full bg-surface border border-surface-lighter rounded-lg px-3 py-2 text-text text-sm font-mono focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors resize-y"
          ></textarea>
          <div class="flex gap-2">
            <button
              onclick={saveEdit}
              class="px-3 py-1.5 bg-primary text-white rounded-lg text-sm font-medium hover:bg-primary-dark transition-colors"
            >
              Save
            </button>
            <button
              onclick={cancelEdit}
              class="px-3 py-1.5 bg-surface-lighter text-text-muted rounded-lg text-sm font-medium hover:text-text transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      {:else}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div
          onclick={() => selectPrompt(prompt.id)}
          role="radio"
          aria-checked={settings.activePromptId === prompt.id}
          tabindex="0"
          class="w-full text-left group relative flex items-center gap-3 px-4 py-3 rounded-xl border transition-all cursor-pointer {settings.activePromptId === prompt.id
            ? 'bg-primary/10 border-primary/30 shadow-sm shadow-primary/10'
            : 'bg-surface-light border-transparent hover:border-surface-lighter'}"
        >
          <div
            class="w-3 h-3 rounded-full border-2 flex-shrink-0 transition-colors {settings.activePromptId === prompt.id
              ? 'border-primary bg-primary'
              : 'border-surface-lighter'}"
          ></div>
          <div class="flex-1 min-w-0">
            <div class="text-sm font-medium text-text">{prompt.name}</div>
            {#if prompt.template}
              <div class="text-xs text-text-muted truncate mt-0.5">{prompt.template.slice(0, 60)}...</div>
            {:else}
              <div class="text-xs text-text-muted/60 italic mt-0.5">No transformation (raw text)</div>
            {/if}
          </div>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onclick={(e) => { e.stopPropagation(); startEdit(prompt); }}
              class="p-1.5 text-text-muted hover:text-text rounded-lg hover:bg-surface-lighter transition-colors"
              aria-label="Edit prompt"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
            </button>
            {#if settings.prompts.length > 1}
              <button
                onclick={(e) => { e.stopPropagation(); deletePrompt(prompt.id); }}
                class="p-1.5 text-text-muted hover:text-danger rounded-lg hover:bg-surface-lighter transition-colors"
                aria-label="Delete prompt"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            {/if}
          </div>
        </div>
      {/if}
    {/each}
  </div>
</div>

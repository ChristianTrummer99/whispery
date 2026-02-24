<script lang="ts">
  import type { Settings } from "$lib/stores";

  let { settings = $bindable(), onSave }: { settings: Settings; onSave: () => void } = $props();
  let inputValue = $state("");

  function addWord() {
    const word = inputValue.trim();
    if (!word || settings.glossary.includes(word)) return;
    settings.glossary = [...settings.glossary, word];
    inputValue = "";
    onSave();
  }

  function removeWord(word: string) {
    settings.glossary = settings.glossary.filter((w) => w !== word);
    onSave();
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      addWord();
    }
  }
</script>

<div class="space-y-4">
  <div>
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-2">
      Custom Vocabulary
    </h3>
    <p class="text-xs text-text-muted mb-4">
      Add words, names, or technical terms that Whisper should recognize correctly.
      These are sent as a prompt hint to improve transcription accuracy (max ~200 words).
    </p>

    <div class="flex gap-2">
      <input
        type="text"
        bind:value={inputValue}
        onkeydown={onKeyDown}
        placeholder="Type a word and press Enter..."
        class="flex-1 bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-sm text-text placeholder-text-muted/50 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
      />
      <button
        onclick={addWord}
        disabled={!inputValue.trim()}
        class="px-4 py-2.5 bg-primary/10 text-primary hover:bg-primary/20 border border-primary/20 rounded-lg text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      >
        Add
      </button>
    </div>
  </div>

  {#if settings.glossary.length > 0}
    <div class="flex flex-wrap gap-2">
      {#each settings.glossary as word (word)}
        <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-surface-light border border-surface-lighter rounded-lg text-sm text-text">
          {word}
          <button
            onclick={() => removeWord(word)}
            aria-label="Remove {word}"
            class="text-text-muted/50 hover:text-danger transition-colors"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </span>
      {/each}
    </div>
    <p class="text-xs text-text-muted">{settings.glossary.length} word{settings.glossary.length === 1 ? "" : "s"}</p>
  {:else}
    <div class="text-center py-6 text-text-muted/50 text-sm">
      No glossary words added yet
    </div>
  {/if}
</div>

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { WHISPER_LANGUAGES, type Settings } from "$lib/stores";
  import { onMount } from "svelte";

  let { settings = $bindable(), onSave }: { settings: Settings; onSave: () => void } = $props();
  let devices = $state<string[]>([]);
  let loading = $state(true);
  let recording = $state(false);
  let collectedKeys = $state<string[]>([]);
  let hotkeyError = $state("");
  let recorderEl = $state<HTMLDivElement | null>(null);

  const CODE_TO_TAURI: Record<string, string> = {
    CapsLock: "CapsLock",
    ScrollLock: "ScrollLock",
    NumLock: "NumLock",
    Backquote: "`", Minus: "-", Equal: "=",
    BracketLeft: "[", BracketRight: "]", Backslash: "\\",
    Semicolon: ";", Quote: "'", Comma: ",", Period: ".", Slash: "/",
    Space: "Space", Enter: "Enter", Backspace: "Backspace", Tab: "Tab",
    Escape: "Escape", Delete: "Delete", Insert: "Insert",
    Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown",
    ArrowUp: "ArrowUp", ArrowDown: "ArrowDown", ArrowLeft: "ArrowLeft", ArrowRight: "ArrowRight",
    AltLeft: "Alt", AltRight: "AltRight",
    ControlLeft: "Control", ControlRight: "ControlRight",
    ShiftLeft: "Shift", ShiftRight: "ShiftRight",
    MetaLeft: "Super", MetaRight: "Super",
    ContextMenu: "ContextMenu", Pause: "Pause", PrintScreen: "PrintScreen",
  };

  const MODIFIER_ORDER = ["CommandOrControl", "Control", "ControlRight", "Super", "Alt", "AltRight", "Shift", "ShiftRight"];
  const MODIFIER_SET = new Set(MODIFIER_ORDER);
  const WINDOWS_LOCK_KEYS = new Set(["CapsLock", "NumLock", "ScrollLock"]);

  function isWindowsPlatform(): boolean {
    return typeof navigator !== "undefined" && /Windows/i.test(navigator.userAgent);
  }

  function codeToTauriKey(code: string): string | null {
    if (/^F\d{1,2}$/.test(code)) return code;
    if (/^Key([A-Z])$/.test(code)) return code.replace("Key", "");
    if (/^Digit(\d)$/.test(code)) return code.replace("Digit", "");
    if (/^Numpad\d$/.test(code)) return code;
    if (CODE_TO_TAURI[code]) return CODE_TO_TAURI[code];
    return null;
  }

  function buildShortcutFromKeys(keys: string[]): string {
    const modifiers = keys.filter((k) => MODIFIER_ORDER.includes(k));
    const nonModifiers = keys.filter((k) => !MODIFIER_ORDER.includes(k));
    const sorted = [
      ...MODIFIER_ORDER.filter((m) => modifiers.includes(m)),
      ...nonModifiers,
    ];
    return sorted.join("+");
  }

  function validateShortcut(keys: string[], mode: Settings["pttMode"]): string | null {
    if (keys.length === 0) return "Please record a shortcut first.";

    const hasNonModifier = keys.some((k) => !MODIFIER_SET.has(k));
    if (!hasNonModifier) {
      return "Shortcut must include at least one non-modifier key (e.g. F1, Space, A).";
    }

    if (isWindowsPlatform() && mode === "hold" && keys.some((k) => WINDOWS_LOCK_KEYS.has(k))) {
      return "Caps/Num/Scroll Lock are unreliable in Hold mode on Windows. Use Toggle mode or a different key.";
    }

    return null;
  }

  const DISPLAY_NAMES: Record<string, string> = {
    CommandOrControl: "Ctrl/Cmd",
    Control: "Ctrl",
    ControlRight: "Right Ctrl",
    Alt: "Alt",
    AltRight: "Right Alt",
    Shift: "Shift",
    ShiftRight: "Right Shift",
    Super: "Cmd/Win",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    CapsLock: "Caps Lock",
    ScrollLock: "Scroll Lock",
    NumLock: "Num Lock",
    Backspace: "⌫",
    Delete: "Del",
    Escape: "Esc",
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
  };

  function displayKey(key: string): string {
    return DISPLAY_NAMES[key] ?? key;
  }

  function formatDisplayKey(shortcut: string): string {
    return shortcut.split("+").map(displayKey).join(" + ");
  }

  function startRecording() {
    collectedKeys = [];
    recording = true;
    requestAnimationFrame(() => recorderEl?.focus());
  }

  function onKeyDown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();

    const key = codeToTauriKey(e.code);
    if (!key) return;
    if (collectedKeys.includes(key)) return;

    collectedKeys = [...collectedKeys, key];
  }

  function removeKey(key: string) {
    collectedKeys = collectedKeys.filter((k) => k !== key);
  }

  function stopRecording() {
    recording = false;
    if (collectedKeys.length > 0) {
      const validationError = validateShortcut(collectedKeys, settings.pttMode);
      if (validationError) {
        hotkeyError = validationError;
        return;
      }
      settings.pttKey = buildShortcutFromKeys(collectedKeys);
      hotkeyError = "";
      onSave();
    }
  }

  function cancelRecording() {
    recording = false;
    collectedKeys = [];
  }

  function setPttMode(mode: Settings["pttMode"]) {
    const keys = settings.pttKey.split("+").filter(Boolean);
    const validationError = validateShortcut(keys, mode);
    if (validationError) {
      hotkeyError = validationError;
      return;
    }

    hotkeyError = "";
    settings.pttMode = mode;
    onSave();
  }

  onMount(async () => {
    try {
      devices = await invoke<string[]>("list_audio_devices");
    } catch (e) {
      console.error("Failed to list devices:", e);
    } finally {
      loading = false;
    }
  });

  async function refreshDevices() {
    loading = true;
    try {
      devices = await invoke<string[]>("list_audio_devices");
    } catch (e) {
      console.error("Failed to refresh devices:", e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-4">
      Microphone
    </h3>

    <div class="flex items-end gap-3">
      <div class="flex-1">
        <label for="audio-device" class="block mb-1 text-sm text-text">Input Device</label>
        {#if loading}
          <div class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text-muted text-sm">
            Scanning devices...
          </div>
        {:else}
          <select
            id="audio-device"
            bind:value={settings.selectedDevice}
            onchange={onSave}
            class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
          >
            <option value="">System Default</option>
            {#each devices as device}
              <option value={device}>{device}</option>
            {/each}
          </select>
        {/if}
      </div>
      <button
        onclick={refreshDevices}
        disabled={loading}
        aria-label="Refresh audio devices"
        class="p-2.5 bg-surface-light border border-surface-lighter rounded-lg text-text-muted hover:text-text hover:border-primary/30 transition-colors disabled:opacity-50"
      >
        <svg class="w-5 h-5" class:animate-spin={loading} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>
    </div>
  </div>

  <hr class="border-surface-lighter" />

  <div>
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-4">
      Push-to-Talk Key
    </h3>
    <label class="block mb-1 text-sm text-text">Shortcut Key</label>
    {#if recording}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <div
        bind:this={recorderEl}
        onkeydown={onKeyDown}
        tabindex="0"
        role="textbox"
        aria-label="Press keys to record shortcut"
        class="w-full bg-surface border-2 border-primary rounded-lg px-4 py-3 focus:outline-none transition-colors"
      >
        {#if collectedKeys.length === 0}
          <p class="text-sm text-primary animate-pulse text-center">Press keys one at a time...</p>
        {:else}
          <div class="flex flex-wrap items-center gap-2">
            {#each collectedKeys as key (key)}
              <span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-primary/15 text-primary border border-primary/25 rounded-md text-sm font-mono">
                {displayKey(key)}
                <button
                  onclick={(e) => { e.stopPropagation(); removeKey(key); }}
                  aria-label="Remove {displayKey(key)}"
                  class="text-primary/50 hover:text-primary transition-colors"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </span>
            {/each}
          </div>
        {/if}
      </div>
      <div class="flex items-center gap-2 mt-3">
        <button
          onclick={stopRecording}
          disabled={collectedKeys.length === 0}
          class="flex items-center gap-1.5 px-4 py-2 bg-primary text-white rounded-lg text-sm font-medium hover:bg-primary-dark transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          Save Shortcut
        </button>
        <button
          onclick={cancelRecording}
          class="px-4 py-2 bg-surface-lighter text-text-muted rounded-lg text-sm font-medium hover:text-text transition-colors"
        >
          Cancel
        </button>
      </div>
    {:else}
      <div class="flex items-center gap-3">
        <div class="flex-1 bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 flex items-center gap-2 min-h-[42px]">
          <div class="flex flex-wrap items-center gap-1.5">
            {#each settings.pttKey.split("+") as key (key)}
              <kbd class="px-2 py-0.5 bg-surface border border-surface-lighter rounded text-sm font-mono text-text">{displayKey(key)}</kbd>
              {#if key !== settings.pttKey.split("+").at(-1)}
                <span class="text-text-muted text-xs">+</span>
              {/if}
            {/each}
          </div>
        </div>
        <button
          onclick={startRecording}
          class="flex items-center gap-1.5 px-4 py-2.5 bg-primary/10 text-primary hover:bg-primary/20 border border-primary/20 rounded-lg text-sm font-medium transition-colors whitespace-nowrap"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
          Change
        </button>
      </div>
    {/if}
    <p class="mt-1.5 text-xs text-text-muted">
      {settings.pttMode === "hold"
        ? "Hold this key combination to record. Release to process."
        : "Press to start recording. Press again to stop and process."}
      Works globally even when the app is not focused.
    </p>
    {#if hotkeyError}
      <p class="mt-2 text-xs text-danger">{hotkeyError}</p>
    {/if}

    <div class="mt-4">
      <label class="block mb-2 text-sm text-text">Hotkey Mode</label>
      <div class="grid grid-cols-2 gap-2">
        <button
          onclick={() => setPttMode("hold")}
          class="flex flex-col items-center gap-1.5 px-4 py-3 rounded-xl border text-sm transition-all {settings.pttMode === 'hold'
            ? 'bg-primary/10 border-primary/30 text-text'
            : 'bg-surface-light border-transparent text-text-muted hover:border-surface-lighter'}"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
          <span class="font-medium">Hold</span>
          <span class="text-[11px] text-text-muted">Hold to record, release to process</span>
        </button>
        <button
          onclick={() => setPttMode("toggle")}
          class="flex flex-col items-center gap-1.5 px-4 py-3 rounded-xl border text-sm transition-all {settings.pttMode === 'toggle'
            ? 'bg-primary/10 border-primary/30 text-text'
            : 'bg-surface-light border-transparent text-text-muted hover:border-surface-lighter'}"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </svg>
          <span class="font-medium">Toggle</span>
          <span class="text-[11px] text-text-muted">Press to start, press again to process</span>
        </button>
      </div>
    </div>
  </div>

  <hr class="border-surface-lighter" />

  <div>
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-4">
      Transcription Language
    </h3>
    <label for="whisper-lang" class="block mb-1 text-sm text-text">Language</label>
    <select
      id="whisper-lang"
      bind:value={settings.whisperLanguage}
      onchange={onSave}
      class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors"
    >
      {#each WHISPER_LANGUAGES as lang}
        <option value={lang.code}>{lang.name}</option>
      {/each}
    </select>
    <p class="mt-1.5 text-xs text-text-muted">
      Helps Whisper produce more accurate transcriptions for your spoken language.
    </p>
  </div>

  <hr class="border-surface-lighter" />

  <div class="space-y-4">
    <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider">
      Output After Transcription
    </h3>

    <label class="flex items-start gap-3 cursor-pointer">
      <input
        type="checkbox"
        bind:checked={settings.autoCopyToClipboard}
        onchange={onSave}
        class="mt-0.5 w-4 h-4 rounded border-surface-lighter text-primary focus:ring-primary/30"
      />
      <div>
        <span class="text-sm text-text">Auto-copy to clipboard</span>
        <p class="text-xs text-text-muted">Copies the final text after every transcription.</p>
      </div>
    </label>

    <label class="flex items-start gap-3 cursor-pointer">
      <input
        type="checkbox"
        bind:checked={settings.autoInsertToInput}
        onchange={onSave}
        class="mt-0.5 w-4 h-4 rounded border-surface-lighter text-primary focus:ring-primary/30"
      />
      <div>
        <span class="text-sm text-text">Auto-insert into focused input</span>
        <p class="text-xs text-text-muted">Sends the final text into whichever field currently has focus.</p>
      </div>
    </label>

    <div class:opacity-50={!settings.autoInsertToInput} class="space-y-1.5">
      <label for="insert-mode" class="block text-sm text-text">Insert mode</label>
      <select
        id="insert-mode"
        bind:value={settings.insertMode}
        onchange={onSave}
        disabled={!settings.autoInsertToInput}
        class="w-full bg-surface-light border border-surface-lighter rounded-lg px-4 py-2.5 text-text focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary/30 transition-colors disabled:cursor-not-allowed"
      >
        <option value="paste_shortcut">Paste shortcut (Cmd/Ctrl + V)</option>
        <option value="type_characters">Type characters (compatibility mode)</option>
      </select>
      <p class="text-xs text-text-muted">
        If some apps react badly to paste shortcuts (for example Softdent), use compatibility mode or turn off auto-insert.
      </p>
    </div>
  </div>

  {#if import.meta.env.DEV}
    <hr class="border-surface-lighter" />

    <div>
      <h3 class="text-sm font-semibold text-text-muted uppercase tracking-wider mb-4">
        Debug
      </h3>
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={settings.saveDebugAudio}
          onchange={onSave}
          class="w-4 h-4 rounded border-surface-lighter text-primary focus:ring-primary/30"
        />
        <div>
          <span class="text-sm text-text">Save audio recordings</span>
          <p class="text-xs text-text-muted">Saves each recording as <code>whispery_debug.wav</code> on your Desktop. The exact bytes sent to Whisper.</p>
        </div>
      </label>
    </div>
  {/if}
</div>

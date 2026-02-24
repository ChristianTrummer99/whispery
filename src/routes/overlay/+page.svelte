<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow, LogicalPosition } from "@tauri-apps/api/window";
  import { load } from "@tauri-apps/plugin-store";
  import { onMount } from "svelte";

  let status = $state<"idle" | "listening" | "processing" | "success" | "error">("idle");
  let audioLevel = $state(0);
  let barHeights = $state([0.15, 0.15, 0.15, 0.15, 0.15]);
  let levelInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      try {
        const store = await load("settings.json");
        const pos = await store.get<{ x: number; y: number }>("overlayPosition");
        if (pos) {
          const win = getCurrentWindow();
          const monitors = await (await import("@tauri-apps/api/window")).availableMonitors();
          const onScreen = monitors.some((m) => {
            const mx = m.position.x;
            const my = m.position.y;
            const mw = m.size.width / m.scaleFactor;
            const mh = m.size.height / m.scaleFactor;
            return pos.x >= mx - 50 && pos.x < mx + mw && pos.y >= my - 50 && pos.y < my + mh;
          });

          if (onScreen) {
            await win.setPosition(new LogicalPosition(pos.x, pos.y));
          } else {
            await store.delete("overlayPosition");
            await store.save();
          }
        }
      } catch { /* position restore is best-effort */ }

      const unlistenFn = await listen<{ state: string; message?: string }>(
        "recording-status",
        (event) => {
          const newStatus = event.payload.state as typeof status;

          if (newStatus === "listening" && status !== "listening") {
            startPollingLevel();
          } else if (newStatus !== "listening") {
            stopPollingLevel();
          }

          status = newStatus;

          if (status === "success" || status === "error") {
            setTimeout(() => {
              status = "idle";
              invoke("hide_overlay_cmd");
            }, 1500);
          }
        }
      );
      unlisten = unlistenFn;
    })();

    return () => {
      unlisten?.();
      stopPollingLevel();
    };
  });

  function startPollingLevel() {
    if (levelInterval) return;
    levelInterval = setInterval(async () => {
      try {
        const level = await invoke<number>("get_audio_level");
        audioLevel = level;
        updateBars(level);
      } catch {
        audioLevel = 0;
      }
    }, 60);
  }

  function stopPollingLevel() {
    if (levelInterval) {
      clearInterval(levelInterval);
      levelInterval = null;
    }
    audioLevel = 0;
    barHeights = [0.15, 0.15, 0.15, 0.15, 0.15];
  }

  function updateBars(level: number) {
    const base = 0.15;
    const jitter = () => (Math.random() - 0.5) * 0.15;
    barHeights = [
      Math.max(base, level * 0.7 + jitter()),
      Math.max(base, level * 1.0 + jitter()),
      Math.max(base, level * 0.85 + jitter()),
      Math.max(base, level * 0.95 + jitter()),
      Math.max(base, level * 0.75 + jitter()),
    ];
  }

  async function startDrag(e: MouseEvent) {
    const win = getCurrentWindow();
    await win.startDragging();

    setTimeout(async () => {
      try {
        const pos = await win.outerPosition();
        const store = await load("settings.json");
        await store.set("overlayPosition", { x: pos.x, y: pos.y });
        await store.save();
      } catch { /* position save is best-effort */ }
    }, 200);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pill-root" onmousedown={startDrag}>
  <div
    class="pill"
    class:listening={status === "listening"}
    class:processing={status === "processing"}
    class:success={status === "success"}
    class:error={status === "error"}
  >
    {#if status === "listening"}
      <svg class="mic-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z"/>
        <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
      </svg>
      <div class="bars">
        {#each barHeights as h, i}
          <div
            class="bar"
            style="height: {Math.min(h, 1) * 100}%; transition-delay: {i * 15}ms"
          ></div>
        {/each}
      </div>
    {:else if status === "processing"}
      <div class="wave-bars">
        {#each Array(7) as _, i}
          <div class="wave-bar" style="animation-delay: {i * 0.1}s"></div>
        {/each}
      </div>
    {:else if status === "success"}
      <svg class="status-icon success-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
      </svg>
      <span class="status-label">Done</span>
    {:else if status === "error"}
      <svg class="status-icon error-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
      </svg>
      <span class="status-label">Error</span>
    {/if}
  </div>
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    padding: 0;
    background: transparent !important;
    overflow: hidden;
  }

  .pill-root {
    width: 220px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: grab;
    user-select: none;
  }

  .pill {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    width: 200px;
    height: 40px;
    border-radius: 9999px;
    background: rgba(40, 40, 40, 0.55);
    border: 1px solid rgba(255, 255, 255, 0.15);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    padding: 0 16px;
    transition: background 0.3s, border-color 0.3s, box-shadow 0.3s;
  }

  .pill.listening {
    border-color: rgba(255, 255, 255, 0.25);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  }

  .pill.processing {
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
  }

  .pill.success {
    border-color: rgba(52, 199, 89, 0.4);
    box-shadow: 0 2px 16px rgba(52, 199, 89, 0.2);
    animation: flash-success 0.4s ease-out;
  }

  .pill.error {
    border-color: rgba(255, 59, 48, 0.4);
    box-shadow: 0 2px 16px rgba(255, 59, 48, 0.2);
    animation: flash-error 0.5s ease-in-out;
  }

  .mic-icon {
    width: 16px;
    height: 16px;
    color: rgba(255, 255, 255, 0.9);
    flex-shrink: 0;
  }

  .bars {
    display: flex;
    align-items: center;
    gap: 3px;
    height: 24px;
    flex: 1;
  }

  .bar {
    width: 4px;
    min-height: 4px;
    background: rgba(255, 255, 255, 0.85);
    border-radius: 2px;
    transition: height 0.08s ease-out;
  }

  .wave-bars {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 24px;
  }

  .wave-bar {
    width: 3px;
    height: 6px;
    background: rgba(255, 255, 255, 0.6);
    border-radius: 2px;
    animation: sine-wave 1s ease-in-out infinite;
  }

  .status-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .success-icon {
    color: #34c759;
    animation: pop 0.4s ease-out;
  }

  .error-icon {
    color: #ff3b30;
    animation: shake 0.5s ease-in-out;
  }

  .status-label {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    font-size: 13px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
  }

  @keyframes sine-wave {
    0%, 100% {
      height: 6px;
      opacity: 0.4;
    }
    50% {
      height: 22px;
      opacity: 0.9;
    }
  }

  @keyframes pop {
    0% { transform: scale(0); }
    60% { transform: scale(1.3); }
    100% { transform: scale(1); }
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    25% { transform: translateX(-3px); }
    75% { transform: translateX(3px); }
  }

  @keyframes flash-success {
    0% { background: rgba(52, 199, 89, 0.15); }
    100% { background: rgba(40, 40, 40, 0.55); }
  }

  @keyframes flash-error {
    0% { background: rgba(255, 59, 48, 0.12); }
    100% { background: rgba(40, 40, 40, 0.55); }
  }
</style>

<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let status = $state<"idle" | "listening" | "processing" | "success" | "error">("idle");
  let message = $state("");

  onMount(() => {
    const unlisten = listen<{ state: string; message?: string }>(
      "recording-status",
      (event) => {
        status = event.payload.state as typeof status;
        message = event.payload.message ?? "";

        if (status === "success" || status === "error") {
          setTimeout(() => {
            status = "idle";
            invoke("hide_overlay_cmd");
          }, 1500);
        }
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  function startDrag() {
    getCurrentWindow().startDragging();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="overlay-root"
  class:listening={status === "listening"}
  class:processing={status === "processing"}
  class:success={status === "success"}
  class:error={status === "error"}
  onmousedown={startDrag}
>
  <div class="orb">
    <div class="ring ring-1"></div>
    <div class="ring ring-2"></div>
    <div class="ring ring-3"></div>
    <div class="core">
      {#if status === "listening"}
        <svg viewBox="0 0 24 24" fill="currentColor" class="icon">
          <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3zm-1-9c0-.55.45-1 1-1s1 .45 1 1v6c0 .55-.45 1-1 1s-1-.45-1-1V5z"/>
          <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z"/>
        </svg>
      {:else if status === "processing"}
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="icon spin">
          <path d="M12 2v4m0 12v4m-7.07-3.93l2.83-2.83m8.48-8.48l2.83-2.83M2 12h4m12 0h4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83"/>
        </svg>
      {:else if status === "success"}
        <svg viewBox="0 0 24 24" fill="currentColor" class="icon pop">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
        </svg>
      {:else if status === "error"}
        <svg viewBox="0 0 24 24" fill="currentColor" class="icon shake">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
      {/if}
    </div>
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
  }

  .overlay-root {
    width: 120px;
    height: 120px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: grab;
    user-select: none;
  }

  .orb {
    position: relative;
    width: 80px;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 2px solid transparent;
    opacity: 0;
    transition: opacity 0.3s, transform 0.3s;
  }

  .listening .ring {
    opacity: 1;
  }

  .listening .ring-1 {
    border-color: #22d3ee;
    animation: pulse 1.5s ease-in-out infinite;
    box-shadow: 0 0 20px #22d3ee60;
  }

  .listening .ring-2 {
    border-color: #6366f180;
    animation: pulse 1.5s ease-in-out infinite 0.3s;
    inset: -8px;
  }

  .listening .ring-3 {
    border-color: #22d3ee40;
    animation: pulse 1.5s ease-in-out infinite 0.6s;
    inset: -16px;
  }

  .processing .ring-1 {
    opacity: 1;
    border-color: transparent;
    border-top-color: #fbbf24;
    border-right-color: #fbbf24;
    animation: spin 0.8s linear infinite;
    box-shadow: 0 0 15px #fbbf2440;
  }

  .processing .ring-2 {
    opacity: 0.6;
    border-color: transparent;
    border-bottom-color: #fbbf2480;
    border-left-color: #fbbf2480;
    animation: spin 1.2s linear infinite reverse;
    inset: -6px;
  }

  .success .ring-1 {
    opacity: 1;
    border-color: #34d399;
    animation: success-ring 0.5s ease-out forwards;
    box-shadow: 0 0 25px #34d39960;
  }

  .error .ring-1 {
    opacity: 1;
    border-color: #f87171;
    box-shadow: 0 0 25px #f8717160;
  }

  .core {
    position: relative;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle, #1e1e2e 0%, #0f0f1a 100%);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    z-index: 1;
  }

  .listening .core {
    background: radial-gradient(circle, #1a2a3e 0%, #0f1a2a 100%);
    box-shadow: 0 0 30px #22d3ee30;
  }

  .processing .core {
    background: radial-gradient(circle, #2a2518 0%, #1a1808 100%);
  }

  .success .core {
    background: radial-gradient(circle, #1a2e28 0%, #0f1a18 100%);
  }

  .error .core {
    background: radial-gradient(circle, #2e1a1a 0%, #1a0f0f 100%);
  }

  .icon {
    width: 24px;
    height: 24px;
    color: #e2e8f0;
  }

  .listening .icon {
    color: #22d3ee;
    animation: breathe 1s ease-in-out infinite;
  }

  .processing .icon {
    color: #fbbf24;
  }

  .success .icon {
    color: #34d399;
  }

  .error .icon {
    color: #f87171;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  .pop {
    animation: pop 0.4s ease-out;
  }

  .shake {
    animation: shake 0.5s ease-in-out;
  }

  @keyframes pulse {
    0%, 100% {
      transform: scale(1);
      opacity: 0.8;
    }
    50% {
      transform: scale(1.15);
      opacity: 0.4;
    }
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes breathe {
    0%, 100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.1);
    }
  }

  @keyframes pop {
    0% {
      transform: scale(0);
    }
    60% {
      transform: scale(1.2);
    }
    100% {
      transform: scale(1);
    }
  }

  @keyframes shake {
    0%, 100% {
      transform: translateX(0);
    }
    25% {
      transform: translateX(-4px);
    }
    75% {
      transform: translateX(4px);
    }
  }

  @keyframes success-ring {
    0% {
      transform: scale(0.8);
      opacity: 0;
    }
    100% {
      transform: scale(1);
      opacity: 1;
    }
  }
</style>

<script lang="ts">
  import { check } from '@tauri-apps/plugin-updater';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(async () => {
    try {
      const update = await check();
      if (!update) return;

      const confirmed = window.confirm(
        `Whispery ${update.version} is available. Install update now?`
      );
      if (!confirmed) return;

      await update.downloadAndInstall();
      window.alert('Update installed. Please restart Whispery to use the new version.');
    } catch (error) {
      console.error('Updater check failed:', error);
    }
  });
</script>

{@render children()}

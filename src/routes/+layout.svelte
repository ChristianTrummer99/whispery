<script lang="ts">
  import { confirm, message } from '@tauri-apps/plugin-dialog';
  import { check } from '@tauri-apps/plugin-updater';
  import { onMount } from 'svelte';

  let { children } = $props();

  onMount(async () => {
    try {
      const update = await check();
      if (!update) return;

      const confirmed = await confirm(
        `Whispery ${update.version} is available. Install update now?`,
        { title: 'Update Available', okLabel: 'Install', cancelLabel: 'Later' }
      );
      if (!confirmed) return;

      await update.downloadAndInstall();
      await message('Update installed. Please restart Whispery to use the new version.', {
        title: 'Update Installed',
        kind: 'info'
      });
    } catch (error) {
      console.error('Updater check failed:', error);
    }
  });
</script>

{@render children()}

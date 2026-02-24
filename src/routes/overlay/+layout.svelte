<script lang="ts">
  let { children } = $props();
</script>

<svelte:head>
  <style>
    *, *::before, *::after {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
    html {
      background: transparent !important;
      background-color: transparent !important;
    }
    body {
      margin: 0;
      padding: 0;
      background: transparent !important;
      background-color: transparent !important;
      overflow: hidden;
    }
  </style>
</svelte:head>

{@render children()}

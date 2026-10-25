<script lang="ts">
  import { i18n } from './lib/i18n.svelte'
  import { app } from './lib/state.svelte'
  import OpenScreen from './screens/OpenScreen.svelte'
  import Shell from './screens/Shell.svelte'

  // A refresh reopens the file without the password when this tab kept the key.
  let resuming = $state(true)
  void app.resume().finally(() => (resuming = false))

  $effect(() => {
    document.documentElement.lang = i18n.lang
  })
</script>

{#if resuming}
  <!-- Nothing to show while the browser copy is reopened. -->
{:else if app.book && app.snapshot}
  <Shell />
{:else}
  <OpenScreen />
{/if}

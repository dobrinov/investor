<script lang="ts">
  import { i18n } from './lib/i18n.svelte'
  import { app } from './lib/state.svelte'
  import OpenScreen from './screens/OpenScreen.svelte'
  import Shell from './screens/Shell.svelte'

  $effect(() => {
    document.documentElement.lang = i18n.lang
  })
</script>

{#if app.book && app.snapshot}
  <Shell />
{:else}
  <OpenScreen />
{/if}

<script lang="ts">
  import { i18n } from './lib/i18n.svelte'
  import { app } from './lib/state.svelte'
  import UpdateNotice from './components/UpdateNotice.svelte'
  import { bridge } from './lib/bridge.svelte'
  import OpenScreen from './screens/OpenScreen.svelte'
  import Shell from './screens/Shell.svelte'

  // A refresh reopens the file without the password when this tab kept the key.
  let resuming = $state(true)
  // `#example` (the marketing page links to it) opens the example portfolio;
  // `#example/trade` and the like come back to a screen after a refresh.
  void app
    .resume()
    .then(async (resumed) => {
      const [first, view] = location.hash.slice(1).split('/')
      if (!resumed && first === 'example') await app.openExample(i18n.lang, view)
    })
    .finally(() => (resuming = false))

  // The desktop app starts the connection for AI tools again if it was on.
  bridge.resume()

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
<UpdateNotice />

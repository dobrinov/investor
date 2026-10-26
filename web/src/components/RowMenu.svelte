<script module lang="ts">
  export interface Item {
    label: string
    run: () => void
    danger?: boolean
  }
</script>

<script lang="ts">
  // The ⋯ button at the end of a row: its actions in a small menu, so a row
  // shows facts and one control instead of a line of buttons.
  import { tick } from 'svelte'

  let { label, items }: { label: string; items: Item[] } = $props()
  let open = $state(false)
  let root = $state<HTMLElement>()
  let menu = $state<HTMLElement>()
  // Fixed to the window, so a table that scrolls sideways cannot clip it.
  let place = $state('')

  function position(): void {
    if (!root) return
    const r = root.getBoundingClientRect()
    const below = window.innerHeight - r.bottom > 220
    place = `right: ${Math.max(8, window.innerWidth - r.right)}px; ${below ? `top: ${r.bottom + 4}px` : `bottom: ${window.innerHeight - r.top + 4}px`}`
  }

  async function toggle(): Promise<void> {
    open = !open
    if (open) {
      position()
      await tick()
      menu?.querySelector<HTMLButtonElement>('button')?.focus()
    }
  }

  function choose(item: Item): void {
    open = false
    item.run()
  }

  function keys(e: KeyboardEvent): void {
    const buttons = [...(menu?.querySelectorAll<HTMLButtonElement>('button') ?? [])]
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement)
    if (e.key === 'Escape') {
      open = false
      root?.querySelector<HTMLButtonElement>('.more')?.focus()
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault()
      buttons[(at + (e.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length]?.focus()
    }
  }

  $effect(() => {
    if (!open) return
    const away = (e: PointerEvent) => {
      if (!root?.contains(e.target as Node)) open = false
    }
    // The menu follows its button when the page scrolls or resizes.
    window.addEventListener('pointerdown', away)
    window.addEventListener('scroll', position, true)
    window.addEventListener('resize', position)
    return () => {
      window.removeEventListener('pointerdown', away)
      window.removeEventListener('scroll', position, true)
      window.removeEventListener('resize', position)
    }
  })
</script>

<span class="wrap" bind:this={root}>
  <button class="more" class:on={open} aria-label={label} aria-haspopup="menu" aria-expanded={open} onclick={toggle}>
    <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5" cy="12" r="1.8" /><circle cx="12" cy="12" r="1.8" /><circle cx="19" cy="12" r="1.8" /></svg>
  </button>
  {#if open}
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div class="menu" role="menu" tabindex="-1" style={place} bind:this={menu} onkeydown={keys}>
      {#each items as item (item.label)}
        <button role="menuitem" class:danger={item.danger} onclick={() => choose(item)}>{item.label}</button>
      {/each}
    </div>
  {/if}
</span>

<style>
  .wrap {
    position: relative;
    display: inline-block;
  }
  .more {
    width: 40px;
    height: 40px;
    display: inline-grid;
    place-items: center;
    border: 1px solid transparent;
    border-radius: 8px;
    background: none;
    color: var(--ink-2);
    cursor: pointer;
  }
  .more:hover,
  .more.on {
    background: var(--accent-soft);
    border-color: var(--line);
  }
  .menu {
    position: fixed;
    z-index: 30;
    min-width: 200px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 10px;
    box-shadow: 0 12px 32px rgb(17 24 33 / 0.16);
    text-align: left;
  }
  .menu button {
    min-height: 40px;
    padding: 0 12px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--ink);
    font: 500 14px var(--sans);
    text-align: left;
    white-space: nowrap;
    cursor: pointer;
  }
  .menu button:hover,
  .menu button:focus-visible {
    background: var(--accent-soft);
    outline: none;
  }
  .menu button.danger {
    color: var(--warn);
  }
</style>

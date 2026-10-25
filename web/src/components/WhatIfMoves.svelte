<script lang="ts">
  // The list of moves tried on top of the plan, and the form that edits one.
  // Every change is saved to the file through the plan.
  import type { Move, MoveKind, Plan } from '../lib/core'
  import { month, whole } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import Help from './Help.svelte'

  let { plan, base, onsave }: { plan: Plan; base: string; onsave: (moves: Move[]) => void } = $props()

  const moves = $derived(plan.what_if ?? [])
  const kinds: MoveKind[] = ['lump', 'spread', 'monthly', 'withdraw']
  const addLabel = { lump: 'proj.addLump', spread: 'proj.addSpread', monthly: 'proj.addMonthly', withdraw: 'proj.addWithdraw' } as const

  // Yearly return and swings, in percent above inflation.
  const presets = {
    deposit: ['0.5', '0'],
    bonds: ['1.5', '6'],
    stocks: ['5', '16'],
    single: ['5', '40'],
  } as const
  type Preset = keyof typeof presets | 'custom'

  let draft = $state<Move | null>(null)
  let preset = $state<Preset>('stocks')

  function blank(kind: MoveKind): Move {
    return {
      id: crypto.randomUUID(),
      name: '',
      enabled: true,
      kind,
      amount: kind === 'monthly' ? '800' : '10000',
      start: null,
      months: 12,
      from_cash: kind === 'lump' || kind === 'spread',
      asset: null,
    }
  }

  function edit(m: Move): void {
    draft = structuredClone($state.snapshot(m)) as Move
    const match = Object.entries(presets).find(([, [r, v]]) => m.asset?.real_return === r && m.asset?.volatility === v)
    preset = m.asset ? ((match?.[0] as Preset) ?? 'custom') : 'stocks'
  }

  function choosePreset(p: Preset): void {
    preset = p
    if (draft?.asset && p !== 'custom') draft.asset = { real_return: presets[p][0], volatility: presets[p][1] }
  }

  const decimal = (v: unknown) => String(v ?? '').trim().replace(',', '.') || '0'

  function save(): void {
    if (!draft) return
    const m = $state.snapshot(draft) as Move
    const clean: Move = {
      ...m,
      name: m.name.trim(),
      amount: decimal(m.amount),
      start: m.start || null,
      months: Math.max(1, Math.round(Number(m.months) || 1)),
      asset: m.asset ? { real_return: decimal(m.asset.real_return), volatility: decimal(m.asset.volatility) } : null,
    }
    const exists = moves.some((x) => x.id === clean.id)
    onsave(exists ? moves.map((x) => (x.id === clean.id ? clean : x)) : [...moves, clean])
    draft = null
  }

  function toggle(m: Move): void {
    onsave(moves.map((x) => (x.id === m.id ? { ...x, enabled: !x.enabled } : x)))
  }

  function remove(m: Move): void {
    onsave(moves.filter((x) => x.id !== m.id))
  }

  /** Adds the same money the other way — all at once or over 12 months — and switches the original off. */
  function compare(m: Move): void {
    const other: Move = { ...$state.snapshot(m), id: crypto.randomUUID(), kind: m.kind === 'lump' ? 'spread' : 'lump', months: 12, enabled: true }
    onsave([...moves.map((x) => (x.id === m.id ? { ...x, enabled: false } : x)), other])
  }

  const title = (m: Move) => m.name || t(`proj.kind.${m.kind}`)
  const when = (m: Move) => (m.start ? month(m.start) : t('proj.now'))

  function summary(m: Move): string {
    const amount = whole(Number(m.amount), base)
    const parts = [t(`proj.sum.${m.kind}`, { amount, when: when(m), months: m.months })]
    if (m.kind === 'lump' || m.kind === 'spread') {
      parts.push(t(m.from_cash ? 'proj.sum.fromCash' : 'proj.sum.newMoney'))
      parts.push(m.asset ? t('proj.sum.asset', { ret: m.asset.real_return, vol: m.asset.volatility }) : t('proj.sum.portfolio'))
    }
    return parts.join(' · ')
  }
</script>

<div class="adders">
  {#each kinds as k (k)}
    <button class="btn ghost" onclick={() => ((draft = blank(k)), (preset = 'stocks'))}>+ {t(addLabel[k])}</button>
  {/each}
</div>

{#if draft}
  {@const d = draft}
  <form class="editor" onsubmit={(e) => (e.preventDefault(), save())}>
    <h3>{t(`proj.kind.${d.kind}`)}</h3>
    <div class="grid fields">
      <label class="field">{t('proj.name')}<input bind:value={d.name} placeholder={t('proj.namePlaceholder')} /></label>
      <label class="field">{t(`proj.amount.${d.kind}`, { currency: base })}<input type="number" min="0" step="100" required bind:value={d.amount} /></label>
      <label class="field">
        {t(d.kind === 'lump' || d.kind === 'withdraw' ? 'proj.when' : 'proj.from')}
        <input type="month" value={d.start ?? ''} oninput={(e) => (d.start = e.currentTarget.value || null)} />
        <span class="muted hint">{t('proj.whenHint')}</span>
      </label>
      {#if d.kind === 'spread'}
        <label class="field">{t('proj.months')}<input type="number" min="1" max="120" bind:value={d.months} /></label>
      {/if}
    </div>

    {#if d.kind === 'lump' || d.kind === 'spread'}
      <fieldset>
        <legend>{t('proj.source')}</legend>
        <label class="choice"><input type="radio" name="source" checked={d.from_cash} onchange={() => (d.from_cash = true)} />{t('proj.fromCash')}</label>
        <label class="choice"><input type="radio" name="source" checked={!d.from_cash} onchange={() => (d.from_cash = false)} />{t('proj.newMoney')}</label>
      </fieldset>
      <fieldset>
        <legend>{t('proj.investIn')}</legend>
        <label class="choice"><input type="radio" name="asset" checked={!d.asset} onchange={() => (d.asset = null)} />{t('proj.likePortfolio')}</label>
        <label class="choice">
          <input
            type="radio"
            name="asset"
            checked={!!d.asset}
            onchange={() => {
              d.asset = { real_return: presets.stocks[0], volatility: presets.stocks[1] }
              preset = 'stocks'
            }}
          />{t('proj.somethingElse')}
        </label>
      </fieldset>
      {#if d.asset}
        {@const a = d.asset}
        <div class="grid fields">
          <label class="field">{t('proj.preset')}
            <select value={preset} onchange={(e) => choosePreset(e.currentTarget.value as Preset)}>
              {#each ['deposit', 'bonds', 'stocks', 'single', 'custom'] as const as p (p)}
                <option value={p}>{t(`proj.preset.${p}`)}</option>
              {/each}
            </select>
          </label>
          <div class="field">
            <span class="with-help"><label for="ret">{t('proj.return')}</label><Help about={t('proj.return')} why={t('proj.returnWhy')} without={t('proj.returnWithout')} /></span>
            <input id="ret" type="number" step="0.5" min="-20" max="30" bind:value={a.real_return} oninput={() => (preset = 'custom')} />
          </div>
          <div class="field">
            <span class="with-help"><label for="vol">{t('proj.volatility')}</label><Help about={t('proj.volatility')} why={t('proj.volatilityWhy')} without={t('proj.volatilityWithout')} /></span>
            <input id="vol" type="number" step="1" min="0" max="100" bind:value={a.volatility} oninput={() => (preset = 'custom')} />
          </div>
        </div>
        <p class="muted small">{t('proj.assetNote')}</p>
      {/if}
      <p class="tip small">{t('proj.lumpVsSpread')}</p>
    {/if}

    <div class="actions">
      <button class="btn" type="submit">{t('proj.save')}</button>
      <button class="btn ghost" type="button" onclick={() => (draft = null)}>{t('proj.cancel')}</button>
    </div>
  </form>
{/if}

{#if moves.length}
  <ul class="moves">
    {#each moves as m (m.id)}
      <li class:off={!m.enabled}>
        <input type="checkbox" checked={m.enabled} onchange={() => toggle(m)} aria-label={t('proj.include', { name: title(m) })} />
        <div class="text">
          <strong>{title(m)}</strong>
          <span class="muted">{summary(m)}</span>
        </div>
        <div class="row-actions">
          {#if m.kind === 'lump' || m.kind === 'spread'}
            <button class="link" onclick={() => compare(m)}>{t(m.kind === 'lump' ? 'proj.trySpread' : 'proj.tryLump')}</button>
          {/if}
          <button class="link" onclick={() => edit(m)}>{t('proj.edit')}</button>
          <button class="link" onclick={() => remove(m)}>{t('proj.remove')}</button>
        </div>
      </li>
    {/each}
  </ul>
{:else if !draft}
  <p class="muted">{t('proj.noMoves')}</p>
{/if}

<style>
  .adders {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .editor {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--surface-2);
  }
  h3 {
    font-size: 16px;
    margin: 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
    gap: 12px;
  }
  .hint {
    font-weight: 400;
    font-size: 12px;
  }
  .with-help {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  fieldset {
    border: 0;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px 20px;
  }
  legend {
    font-size: 13px;
    font-weight: 500;
    color: var(--ink-2);
    margin-bottom: 6px;
    width: 100%;
  }
  .choice {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
    font-size: 14px;
  }
  .tip {
    margin: 0;
    padding: 10px 14px;
    border-radius: 8px;
    background: var(--accent-soft);
    color: var(--ink-2);
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .moves {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .moves li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--line-soft);
    border-radius: 10px;
  }
  .moves li.off .text {
    opacity: 0.55;
  }
  .moves input[type='checkbox'] {
    width: 20px;
    height: 20px;
    accent-color: var(--accent);
  }
  .text {
    flex: 1 1 260px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 14px;
  }
  .row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 14px;
  }
  .link {
    border: 0;
    background: none;
    padding: 6px 0;
    color: var(--accent);
    font: 500 13px var(--sans);
    cursor: pointer;
  }
  .link:hover {
    text-decoration: underline;
  }
</style>

<script lang="ts">
  import RowMenu from '../components/RowMenu.svelte'
  import Help from '../components/Help.svelte'
  import PlanVsActual from '../components/PlanVsActual.svelte'
  import type { Goal, Plan, Profile } from '../lib/core'
  import { money } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const thisYear = new Date().getFullYear()

  type Key = 'birth' | 'spending' | 'investing' | 'risk' | 'retirementAge' | 'retirementSpending' | 'pensionAge' | 'pension'
  interface Field {
    key: Key
    prop: keyof Profile
    kind: 'year' | 'age' | 'money' | 'risk'
    label: () => string
  }

  const fields: Field[] = [
    { key: 'birth', prop: 'birth_year', kind: 'year', label: () => t('goals.birthYear') },
    { key: 'spending', prop: 'monthly_spending', kind: 'money', label: () => t('goals.spending', { currency: base }) },
    { key: 'investing', prop: 'monthly_investing', kind: 'money', label: () => t('goals.investing', { currency: base }) },
    { key: 'risk', prop: 'risk', kind: 'risk', label: () => t('goals.risk') },
    { key: 'retirementAge', prop: 'retirement_age', kind: 'age', label: () => t('goals.retirementAge') },
    { key: 'retirementSpending', prop: 'retirement_spending', kind: 'money', label: () => t('goals.retirementSpending', { currency: base }) },
    { key: 'pensionAge', prop: 'pension_age', kind: 'age', label: () => t('goals.pensionAge') },
    { key: 'pension', prop: 'monthly_pension', kind: 'money', label: () => t('goals.pension', { currency: base }) },
  ]

  const empty: Profile = {
    birth_year: null,
    monthly_spending: null,
    monthly_investing: null,
    risk: 'balanced',
    retirement_age: null,
    pension_age: null,
    monthly_pension: null,
    retirement_spending: null,
  }

  // Editable copy; saved to the file on "Save". Empty fields are allowed.
  let profile = $state<Record<string, unknown>>({ ...empty, ...(app.snapshot?.plan.profile ?? {}) })
  let editingProfile = $state(!app.snapshot?.plan.profile)
  let draft = $state<Goal | null>(null)

  const cash = $derived(
    s.trial_balance.filter((r) => r.account.startsWith('Assets:Cash:')).reduce((sum, r) => sum + Number(r.debit) - Number(r.credit), 0),
  )

  const statuses = $derived.by(() => {
    void app.snapshot
    return new Map((app.book?.goals(thisYear, 'base') ?? []).map((g) => [g.id, g]))
  })

  async function save(plan: Plan): Promise<void> {
    await app.setPlan(plan)
  }

  // Amounts travel to the core as exact decimal strings; blanks mean "not given".
  const decimal = (v: unknown) => String(v ?? '0').trim().replace(',', '.') || '0'
  const optionalDecimal = (v: unknown) => {
    const text = String(v ?? '').trim().replace(',', '.')
    return text === '' ? null : text
  }
  const optionalNumber = (v: unknown) => (v === '' || v === null || v === undefined || Number.isNaN(Number(v)) ? null : Number(v))

  async function saveProfile(): Promise<void> {
    const p = $state.snapshot(profile)
    const next = { ...empty, risk: (p.risk as Profile['risk']) ?? 'balanced' } as Profile
    for (const f of fields) {
      if (f.kind === 'money') (next[f.prop] as unknown) = optionalDecimal(p[f.prop])
      else if (f.kind !== 'risk') (next[f.prop] as unknown) = optionalNumber(p[f.prop])
    }
    await save({ ...s.plan, profile: next })
    editingProfile = false
  }

  function shown(f: Field, p: Profile): string {
    const v = p[f.prop]
    if (v === null || v === undefined || v === '') return t('goals.notGiven')
    if (f.kind === 'risk') return t(`risk.${p.risk}`)
    if (f.kind === 'money') return money(v as string, base)
    if (f.kind === 'year') return `${v} · ${t('goals.ageNow', { age: thisYear - Number(v) })}`
    return String(v)
  }

  function newGoal(kind: Goal['kind']): void {
    draft =
      kind === 'emergency_fund'
        ? { id: crypto.randomUUID(), name: t('goals.emergencyName'), kind, target: '3', year: null, monthly: '0' }
        : { id: crypto.randomUUID(), name: '', kind, target: '10000', year: thisYear + 5, monthly: '200' }
  }

  async function saveGoal(): Promise<void> {
    if (!draft) return
    const goal = { ...$state.snapshot(draft), target: decimal(draft.target), monthly: decimal(draft.monthly) }
    const others = s.plan.goals.filter((g) => g.id !== goal.id)
    await save({ ...s.plan, goals: [...others, goal] })
    draft = null
  }

  async function remove(goal: Goal): Promise<void> {
    if (!confirm(t('goals.removeConfirm', { name: goal.name }))) return
    await save({ ...s.plan, goals: s.plan.goals.filter((g) => g.id !== goal.id) })
  }

  const percent = (v: number | null) => (v === null ? '—' : `${Math.round(v * 100)}%`)
</script>

<header>
  <h1>{t('nav.goals')}</h1>
  <p class="muted">{t('goals.subtitle')}</p>
</header>

<section class="goals" aria-label={t('goals.list')}>
  {#each s.plan.goals as goal (goal.id)}
    {@const st = statuses.get(goal.id)}
    <article class="card goal">
      <div class="head">
        <h2>{goal.name}</h2>
        {#if st}<span class="pill" class:warn={!st.on_track}>{st.on_track ? t('goals.onTrack') : t('goals.behind')}</span>{/if}
        <RowMenu
          label={t('common.moreFor', { name: goal.name })}
          items={[
            { label: t('common.edit'), run: () => (draft = { ...goal }) },
            { label: t('common.remove'), run: () => remove(goal), danger: true },
          ]}
        />
      </div>
      <p class="muted when">
        {goal.kind === 'emergency_fund' ? t('goals.emergencyWhen', { months: goal.target }) : t('goals.savingsWhen', { year: goal.year ?? '—' })}
      </p>
      {#if st?.missing}
        <p class="notice">{t('goals.needsSpending')}</p>
      {:else if st}
        <div class="bar" role="img" aria-label={t('goals.progress', { pct: Math.round((st.have / Math.max(1, st.target)) * 100) })}>
          <span style:width="{Math.min(100, (st.have / Math.max(1, st.target)) * 100)}%" class:warn={!st.on_track}></span>
        </div>
        <p class="num amounts">{money(st.have, base)} / {money(st.target, base)}</p>
        {#if goal.kind === 'savings'}
          <dl class="facts small">
            <div><dt>{t('goals.chance')}</dt><dd class="num">{percent(st.chance)}</dd></div>
            <div><dt>{t('goals.needed')}</dt><dd class="num">{money(st.needed_monthly ?? 0, base)}</dd></div>
            <div><dt>{t('goals.putIn')}</dt><dd class="num">{money(goal.monthly, base)}</dd></div>
          </dl>
        {/if}
      {/if}
    </article>
  {:else}
    <p class="muted">{t('goals.none')}</p>
  {/each}
</section>

{#if draft}
  <section class="card">
    <h2>{draft.name || t('goals.newGoal')}</h2>
    <form class="grid fields" onsubmit={(e) => (e.preventDefault(), saveGoal())}>
      <label class="field">{t('goals.name')}<input bind:value={draft.name} required /></label>
      {#if draft.kind === 'emergency_fund'}
        <label class="field">{t('goals.months')}<input type="number" min="1" max="24" bind:value={draft.target} required /></label>
      {:else}
        <label class="field">{t('goals.target', { currency: base })}<input inputmode="decimal" bind:value={draft.target} required /></label>
        <label class="field">{t('goals.byYear')}<input type="number" min={thisYear} max={thisYear + 60} bind:value={draft.year} required /></label>
        <label class="field">{t('goals.monthly', { currency: base })}<input inputmode="decimal" bind:value={draft.monthly} required /></label>
      {/if}
      <div class="actions">
        <button class="btn">{t('common.save')}</button>
        <button type="button" class="btn ghost" onclick={() => (draft = null)}>{t('common.cancel')}</button>
      </div>
    </form>
  </section>
{:else}
  <div class="row">
    <button class="btn" onclick={() => newGoal('savings')}>{t('goals.addSavings')}</button>
    {#if !s.plan.goals.some((g) => g.kind === 'emergency_fund')}
      <button class="btn ghost" onclick={() => newGoal('emergency_fund')}>{t('goals.addEmergency')}</button>
    {/if}
  </div>
{/if}

<PlanVsActual plan={s.plan} {base} {cash} planned={statuses} />

<section class="card">
  <div class="head">
    <h2>{t('goals.profile')}</h2>
    {#if !editingProfile}<button class="btn ghost" onclick={() => (editingProfile = true)}>{t('common.edit')}</button>{/if}
  </div>
  {#if editingProfile}
    <form class="grid fields" onsubmit={(e) => (e.preventDefault(), saveProfile())}>
      {#each fields as f (f.key)}
        <div class="field">
          <div class="field-head">
            <label for="profile-{f.key}">{f.label()}</label>
            <Help about={f.label()} why={t(`help.${f.key}.why`)} without={t(`help.${f.key}.without`)} />
          </div>
          {#if f.kind === 'risk'}
            <select id="profile-{f.key}" bind:value={profile.risk}>
              <option value="cautious">{t('risk.cautious')}</option>
              <option value="balanced">{t('risk.balanced')}</option>
              <option value="growth">{t('risk.growth')}</option>
            </select>
          {:else if f.kind === 'money'}
            <input id="profile-{f.key}" inputmode="decimal" placeholder={t('goals.optional')} bind:value={profile[f.prop]} />
          {:else}
            <input
              id="profile-{f.key}"
              type="number"
              placeholder={t('goals.optional')}
              min={f.kind === 'year' ? 1920 : 18}
              max={f.kind === 'year' ? thisYear : 90}
              bind:value={profile[f.prop]}
            />
          {/if}
        </div>
      {/each}
      <p class="muted small all">{t('goals.allOptional')}</p>
      <div class="actions"><button class="btn">{t('common.save')}</button></div>
    </form>
  {:else if s.plan.profile}
    {@const p = s.plan.profile}
    <dl class="facts">
      {#each fields as f (f.key)}
        <div>
          <dt>{f.label()} <Help about={f.label()} why={t(`help.${f.key}.why`)} without={t(`help.${f.key}.without`)} /></dt>
          <dd class:num={f.kind === 'money'} class:missing={p[f.prop] === null || p[f.prop] === undefined}>{shown(f, p)}</dd>
        </div>
      {/each}
    </dl>
  {/if}
</section>

<p class="muted small">{t('goals.method')}</p>

<style>
  header {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  h1 {
    font-size: 28px;
  }
  h2 {
    font-size: 18px;
  }
  header p,
  p {
    margin: 0;
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 14px;
  }
  .field-head {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 22px;
  }
  .all {
    grid-column: 1 / -1;
  }
  .facts dt {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .missing {
    color: var(--muted);
    font-weight: 400;
  }
  .actions {
    grid-column: 1 / -1;
    display: flex;
    gap: 8px;
  }
  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 32px;
    margin: 0;
  }
  .facts dt {
    font-size: 12px;
    color: var(--muted);
  }
  .facts dd {
    margin: 2px 0 0;
    font-size: 16px;
    font-weight: 500;
  }
  .facts.small dd {
    font-size: 15px;
  }
  .goals {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 16px;
  }
  .goal {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .goal .head {
    margin-bottom: 0;
  }
  .goal .head h2 {
    flex: 1;
  }
  .when {
    font-size: 13px;
  }
  .bar {
    height: 8px;
    background: var(--line-soft);
    border-radius: 4px;
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--accent);
  }
  .bar span.warn {
    background: #c77a2e;
  }
  .amounts {
    font-size: 14px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
  }
</style>

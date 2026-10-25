<script lang="ts">
  import type { Goal, Plan, Profile } from '../lib/core'
  import { money } from '../lib/format'
  import { t } from '../lib/i18n.svelte'
  import { app } from '../lib/state.svelte'

  const s = $derived(app.snapshot!)
  const base = $derived(s.settings.base_currency)
  const thisYear = new Date().getFullYear()

  const defaults: Profile = {
    birth_year: thisYear - 35,
    monthly_spending: '1500',
    monthly_investing: '300',
    risk: 'balanced',
    retirement_age: 65,
    pension_age: 65,
    monthly_pension: '0',
    retirement_spending: '1500',
  }

  // Editable copies; saved to the file on "Save".
  let profile = $state<Profile>({ ...defaults, ...(app.snapshot?.plan.profile ?? {}) })
  let editingProfile = $state(!app.snapshot?.plan.profile)
  let draft = $state<Goal | null>(null)

  const statuses = $derived.by(() => {
    void app.snapshot
    return new Map((app.book?.goals(thisYear, 'base') ?? []).map((g) => [g.id, g]))
  })

  async function save(plan: Plan): Promise<void> {
    await app.setPlan(plan)
  }

  // Amounts travel to the core as exact decimal strings.
  const decimal = (v: unknown) => String(v ?? '0').trim().replace(',', '.') || '0'

  async function saveProfile(): Promise<void> {
    const p = $state.snapshot(profile)
    await save({
      ...s.plan,
      profile: {
        ...p,
        monthly_spending: decimal(p.monthly_spending),
        monthly_investing: decimal(p.monthly_investing),
        monthly_pension: decimal(p.monthly_pension),
        retirement_spending: decimal(p.retirement_spending),
      },
    })
    editingProfile = false
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

<section class="card">
  <div class="head">
    <h2>{t('goals.profile')}</h2>
    {#if !editingProfile}<button class="btn ghost" onclick={() => (editingProfile = true)}>{t('common.edit')}</button>{/if}
  </div>
  {#if editingProfile}
    <form class="grid" onsubmit={(e) => (e.preventDefault(), saveProfile())}>
      <label class="field">{t('goals.birthYear')}<input type="number" min="1920" max={thisYear} bind:value={profile.birth_year} required /></label>
      <label class="field">{t('goals.spending', { currency: base })}<input inputmode="decimal" bind:value={profile.monthly_spending} required /></label>
      <label class="field">{t('goals.investing', { currency: base })}<input inputmode="decimal" bind:value={profile.monthly_investing} required /></label>
      <label class="field">{t('goals.risk')}
        <select bind:value={profile.risk}>
          <option value="cautious">{t('risk.cautious')}</option>
          <option value="balanced">{t('risk.balanced')}</option>
          <option value="growth">{t('risk.growth')}</option>
        </select>
      </label>
      <label class="field">{t('goals.retirementAge')}<input type="number" min="40" max="80" bind:value={profile.retirement_age} required /></label>
      <label class="field">{t('goals.retirementSpending', { currency: base })}<input inputmode="decimal" bind:value={profile.retirement_spending} required /></label>
      <label class="field">{t('goals.pensionAge')}<input type="number" min="50" max="80" bind:value={profile.pension_age} required /></label>
      <label class="field">{t('goals.pension', { currency: base })}<input inputmode="decimal" bind:value={profile.monthly_pension} required /></label>
      <div class="actions"><button class="btn">{t('common.save')}</button></div>
    </form>
  {:else if s.plan.profile}
    {@const p = s.plan.profile}
    <dl class="facts">
      <div><dt>{t('goals.age')}</dt><dd>{thisYear - p.birth_year}</dd></div>
      <div><dt>{t('goals.spendingShort')}</dt><dd class="num">{money(p.monthly_spending, base)}</dd></div>
      <div><dt>{t('goals.investingShort')}</dt><dd class="num">{money(p.monthly_investing, base)}</dd></div>
      <div><dt>{t('goals.risk')}</dt><dd>{t(`risk.${p.risk}`)}</dd></div>
      <div><dt>{t('goals.retirementAge')}</dt><dd>{p.retirement_age}</dd></div>
    </dl>
  {/if}
</section>

<section class="goals" aria-label={t('goals.list')}>
  {#each s.plan.goals as goal (goal.id)}
    {@const st = statuses.get(goal.id)}
    <article class="card goal">
      <div class="head">
        <h2>{goal.name}</h2>
        {#if st}<span class="pill" class:warn={!st.on_track}>{st.on_track ? t('goals.onTrack') : t('goals.behind')}</span>{/if}
      </div>
      <p class="muted when">
        {goal.kind === 'emergency_fund' ? t('goals.emergencyWhen', { months: goal.target }) : t('goals.savingsWhen', { year: goal.year ?? '—' })}
      </p>
      {#if st}
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
      <div class="row">
        <button class="btn ghost" onclick={() => (draft = { ...goal })}>{t('common.edit')}</button>
        <button class="btn ghost" onclick={() => remove(goal)}>{t('common.remove')}</button>
      </div>
    </article>
  {:else}
    <p class="muted">{t('goals.none')}</p>
  {/each}
</section>

{#if draft}
  <section class="card">
    <h2>{draft.name || t('goals.newGoal')}</h2>
    <form class="grid" onsubmit={(e) => (e.preventDefault(), saveGoal())}>
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

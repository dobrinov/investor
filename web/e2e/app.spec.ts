import { expect, test } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('investor.language', 'en'))
})

test('the overview shows net worth and switches between kinds of holdings', async ({ page }) => {
  await page.goto('/app/#example')
  await expect(page.getByRole('heading', { name: 'Net worth' })).toBeVisible()
  await expect(page.getByRole('img', { name: /Stocks & funds/ })).toBeVisible()
  await page.getByRole('tab', { name: /Banks/ }).click()
  await expect(page.getByRole('heading', { name: 'Bank accounts' })).toBeVisible()
  await page.getByRole('tab', { name: /Stocks & funds/ }).click()
  await expect(page.getByRole('heading', { name: 'Holdings' })).toBeVisible()
})

test('a row menu opens the price editor', async ({ page }) => {
  await page.goto('/app/#example')
  await page.getByRole('button', { name: 'More for AAPL' }).click()
  await page.getByRole('menuitem', { name: 'Update price' }).click()
  await expect(page.getByRole('menu')).toHaveCount(0)
  await expect(page.getByRole('textbox', { name: /AAPL/ })).toBeVisible()
})

test('settings has the language picker', async ({ page }) => {
  await page.goto('/app/#example/settings')
  await page.getByRole('combobox', { name: 'Language' }).selectOption('de')
  await expect(page.getByRole('heading', { name: 'Einstellungen' })).toBeVisible()
})

test.describe('on a phone', () => {
  test.use({ viewport: { width: 390, height: 844 } })

  test('nothing is wider than the screen', async ({ page }) => {
    for (const path of ['/app/#example', '/app/#example/trade', '/app/#example/tax', '/']) {
      await page.goto(path)
      await page.waitForLoadState('networkidle')
      const [width, scroll] = await page.evaluate(() => [innerWidth, document.documentElement.scrollWidth])
      expect(scroll, path).toBeLessThanOrEqual(width)
    }
  })

  test('the top bar sits above the content', async ({ page }) => {
    await page.goto('/app/#example')
    const bar = await page.locator('header.topbar').boundingBox()
    expect(bar?.width).toBe(390)
  })
})

test('your own AI tool can read the file and record a change you approve', async ({ page }) => {
  // Stands in for the desktop app: hands the web view MCP requests and keeps its replies.
  await page.addInitScript(() => {
    const jobs = [
      { id: 1, message: { jsonrpc: '2.0', id: 'a', method: 'initialize', params: { protocolVersion: '2025-06-18', clientInfo: { name: 'test-tool' } } } },
      { id: 2, message: { jsonrpc: '2.0', id: 'b', method: 'tools/call', params: { name: 'portfolio', arguments: {} } } },
      { id: 3, message: { jsonrpc: '2.0', id: 'c', method: 'tools/call', params: { name: 'record_trade', arguments: { side: 'buy', broker: 'IBKR', symbol: 'VWCE', units: '10', price: '131.40', commission: '1.25' } } } },
    ]
    const w = window as unknown as Record<string, unknown>
    const replies: Record<number, unknown> = {}
    w.__replies = replies
    w.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: { id: number; reply: unknown }) => {
        if (cmd === 'bridge_next') {
          await new Promise((r) => setTimeout(r, 100))
          return jobs.shift() ?? (await new Promise((r) => setTimeout(r, 2000)), null)
        }
        if (cmd === 'bridge_reply') replies[args.id] = args.reply
        return null
      },
    }
  })
  await page.goto('/app/#example')
  await page.getByRole('button', { name: 'Assistant' }).click()
  await page.getByRole('tab', { name: /Your AI tool/ }).click()
  await page.getByRole('button', { name: 'Turn on' }).click()
  await expect(page.getByText('test-tool wants to:')).toBeVisible()
  await expect(page.locator('#assistant').getByText('Record a trade')).toBeVisible()
  const replies = () => page.evaluate(() => (window as unknown as { __replies: Record<number, { result?: { content: { text: string }[] } }> }).__replies)
  expect(Object.keys(await replies())).toEqual(['1', '2'])
  await page.getByRole('button', { name: 'Approve' }).click()
  await expect.poll(async () => (await replies())[3]?.result?.content[0].text ?? '').toContain('"recorded": true')
  await expect(page.getByRole('row', { name: /VWCE/ }).getByRole('cell').nth(1)).toHaveText('55')
})

test('the tax screen says what to type into the return, with holdings at year end', async ({ page }) => {
  await page.goto('/app/#example/tax')
  const declaration = page.locator('.declaration')
  await expect(declaration.getByRole('heading', { name: 'Your declaration for 2026' })).toBeVisible()
  const holdings = declaration.locator('.form-part', { hasText: 'Приложение 8, Част I' }).filter({ hasNotText: 'Част III' })
  await expect(holdings.getByRole('cell', { name: /Vanguard FTSE All-World/ })).toBeVisible()
  await expect(holdings.getByText('Bitcoin')).toHaveCount(0)
  await expect(declaration.locator('.form-part', { hasText: 'code 508' }).first().getByRole('button', { name: /Copy 1044.00/ })).toBeVisible()
})

test('entries, tax notes and the example read in the chosen language', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('investor.language', 'bg'))
  await page.goto('/app/#example')
  await expect(page.getByText('Продажба 10 VWCE за 1280 EUR')).toBeVisible()
  await page.getByRole('tab', { name: /Банки/ }).click()
  await expect(page.getByRole('cell', { name: 'Спестовна сметка' }).first()).toBeVisible()
  await page.getByRole('navigation', { name: 'Основна навигация' }).or(page.locator('aside nav')).first().getByRole('button', { name: 'Данъци и отчитане' }).click()
  await expect(page.getByText(/Работен вариант\. Капиталови печалби/)).toBeVisible()
})

test('every description the app writes has a translation', async ({ page }) => {
  // Fails when the core's wording changes and the patterns in describe.ts no longer match.
  await page.addInitScript(() => localStorage.setItem('investor.language', 'bg'))
  await page.goto('/app/#example')
  await page.locator('aside nav').getByRole('button', { name: 'Главна книга' }).click()
  const lines = await page.locator('main strong').allTextContents()
  expect(lines.length).toBeGreaterThan(10)
  const english = lines.filter((l) => /^(Buy|Sell|Deposit|Withdrawal|Dividend|Withholding|Fee|Interest|Convert|Bought|Sold|Opening balance)\b|: (opening balance|balance now|paid|owed)\b/.test(l))
  expect(english).toEqual([])
})

test('a debt comes off net worth and shows when it will be paid off', async ({ page }) => {
  await page.goto('/app/#example')
  await expect(page.getByRole('term').filter({ hasText: 'Debt' })).toBeVisible()
  await page.getByRole('tab', { name: /Debt/ }).click()
  const row = page.getByRole('row', { name: /Car loan/ })
  await expect(row).toContainText('6.9%')
  await expect(row).toContainText('€8,400.00')
  await expect(row).toContainText(/\d+ y \d+ m/)
})

test('crypto sales get their own part of the declaration', async ({ page }) => {
  await page.goto('/app/#example/tax')
  const crypto = page.locator('.declaration .form-part', { hasText: 'code 5082' })
  await expect(crypto.getByRole('row')).toHaveCount(2)
  await expect(page.getByRole('row', { name: /BTC/ }).getByText('Crypto')).toBeVisible()
  await expect(page.getByRole('row', { name: /BTC/ }).getByText('Taxable')).toBeVisible()
})

test('on the website the CFO explains it needs the desktop app', async ({ page }) => {
  await page.goto('/app/#example')
  await page.locator('aside nav').getByRole('button', { name: 'Your CFO' }).click()
  await expect(page.getByRole('link', { name: 'Download the desktop app' })).toBeVisible()
})

test('your CFO reads the whole picture, briefs you and answers follow-ups', async ({ page }) => {
  // Stands in for the desktop app with a model on this computer.
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>
    const sent: string[] = []
    w.__sent = sent
    const briefing = {
      headline: 'You are in good shape, but the car loan costs more than your investments earn.',
      summary: 'Net worth is healthy and the emergency fund is covered.',
      actions: [
        { title: 'Pay down the car loan', why: 'It costs 6.9% a year.', impact: 'A sure 6.9% return', priority: 'now' },
        { title: 'Invest idle cash', why: 'Cash above the emergency fund loses value.', impact: 'Growth over time', priority: 'soon' },
      ],
      watch: ['A large share in one fund'],
      questions: ['Do I want to keep the car loan?'],
    }
    w.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: { request?: { method: string; url: string; body?: { messages: { content: string }[] } } }) => {
        if (cmd === 'bridge_next') return await new Promise((r) => setTimeout(() => r(null), 2000))
        if (cmd !== 'model_request') return null
        const req = args.request!
        if (req.method === 'GET') return { status: 200, body: req.url.includes('11434') ? JSON.stringify({ models: [{ name: 'fake:latest' }] }) : '{}' }
        const text = req.body!.messages.map((m) => m.content).join('\n')
        sent.push(text)
        const content = text.includes('Write a short briefing') ? '```json\n' + JSON.stringify(briefing) + '\n```' : 'Paying it down first is a sure return; keep the emergency fund intact.'
        return { status: 200, body: JSON.stringify({ choices: [{ message: { content } }] }) }
      },
    }
  })
  await page.goto('/app/#example')
  await page.locator('aside nav').getByRole('button', { name: 'Your CFO' }).click()
  await page.getByRole('button', { name: 'Find models on this computer' }).click()
  await page.getByRole('button', { name: /Use fake:latest/ }).click()
  await expect(page.getByText('Nothing leaves this computer.')).toBeVisible()
  await page.getByRole('button', { name: 'Get a briefing' }).click()
  await expect(page.getByRole('heading', { name: /car loan costs more/ })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Pay down the car loan' })).toBeVisible()
  const sent = () => page.evaluate(() => (window as unknown as { __sent: string[] }).__sent)
  // The model gets the app's own figures, including the signals.
  expect((await sent())[0]).toContain('"high_interest_debt"')
  await page.getByRole('button', { name: 'Ask about this' }).first().click()
  await expect(page.getByText('Paying it down first is a sure return')).toBeVisible()
  expect((await sent())[1]).toContain('virtual CFO')
})

test('any password is allowed, with advice on how strong it is', async ({ page }) => {
  await page.goto('/app/')
  await page.getByRole('button', { name: 'Create a new file' }).click()
  const password = page.getByLabel('Password', { exact: true })
  await password.fill('password')
  await expect(page.getByText('Very weak')).toBeVisible()
  await expect(page.getByText(/common password/)).toBeVisible()
  await password.fill('river lamp quiet orbit')
  await expect(page.getByText(/Strong|Very strong/).first()).toBeVisible()
  await password.fill('abc')
  await page.getByLabel('Repeat password').fill('abc')
  await expect(page.getByText('Use at least 10 characters.')).toHaveCount(0)
})

test('your CFO can use Claude Code with your own sign-in', async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>
    const asked: { system: string; prompt: string }[] = []
    w.__asked = asked
    w.__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: { system: string; prompt: string; request?: { method: string } }) => {
        if (cmd === 'bridge_next') return await new Promise((r) => setTimeout(() => r(null), 2000))
        if (cmd === 'claude_code_path') return '/Users/me/.local/bin/claude'
        if (cmd === 'model_request') return { status: 404, body: '' }
        if (cmd === 'claude_code_ask') {
          asked.push(args)
          return JSON.stringify({ headline: 'The car loan comes first.', summary: '', actions: [{ title: 'Pay down the car loan', why: '6.9%', impact: 'sure return', priority: 'now' }], watch: [], questions: [] })
        }
        return null
      },
    }
  })
  await page.goto('/app/#example')
  await page.locator('aside nav').getByRole('button', { name: 'Your CFO' }).click()
  await page.getByRole('button', { name: 'Find models on this computer' }).click()
  await page.getByRole('button', { name: /Use Claude Code/ }).click()
  await expect(page.getByText('Your figures are sent to Anthropic (Claude Code).')).toBeVisible()
  await page.getByRole('button', { name: 'Get a briefing' }).click()
  await expect(page.getByRole('heading', { name: 'The car loan comes first.' })).toBeVisible()
  const asked = await page.evaluate(() => (window as unknown as { __asked: { system: string; prompt: string }[] }).__asked)
  expect(asked[0].system).toContain('virtual CFO')
  expect(asked[0].prompt).toContain('"high_interest_debt"')
})

test('crypto has its own tab and slice of net worth', async ({ page }) => {
  await page.goto('/app/#example')
  await expect(page.getByRole('img', { name: /Crypto/ })).toBeVisible()
  await expect(page.getByRole('row', { name: /BTC/ })).toHaveCount(0)
  await page.getByRole('tab', { name: /Crypto/ }).click()
  await expect(page.getByRole('row', { name: /BTC/ })).toBeVisible()
  await expect(page.getByRole('row', { name: /VWCE/ })).toHaveCount(0)
})

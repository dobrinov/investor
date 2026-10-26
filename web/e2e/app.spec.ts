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
  const english = lines.filter((l) => /^(Buy|Sell|Deposit|Withdrawal|Dividend|Withholding|Fee|Interest|Convert|Bought|Sold|Opening balance)\b|: (opening balance|balance now|paid)\b/.test(l))
  expect(english).toEqual([])
})

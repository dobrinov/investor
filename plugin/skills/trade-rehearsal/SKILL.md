---
name: trade-rehearsal
description: Rehearse a purchase or sale in the person's Investor portfolio before they make it — fees, cash left at the broker, gain and tax on a sale, the emergency fund, concentration and the 20-year effect — and compare it with sensible alternatives. Use when they ask "should I buy/sell…", "what if I put X into Y", or want to check a trade.
---

# Rehearse a trade

Use `preview_trade` from the `investor` tools; it records nothing. Only
`record_trade` writes, and only after the person agrees.

## Steps

1. Call `portfolio` to learn the brokers, holdings, cash and currencies.
   If the instrument, broker, units or amount are unclear, ask once.
2. Call `preview_trade` with what the person described. Use the venue they
   will trade on (it can decide the tax, e.g. EU regulated markets are
   tax-free in Bulgaria) and include their broker's fees if they know them.
3. Read from the result:
   - the cost or proceeds and the fees, and the fee as a share of the trade;
   - cash at the broker before and after, and any deposit needed;
   - for a sale: the FIFO gain, its tax treatment and the estimated tax;
   - holdings before and after, and whether one holding passes half;
   - all money in 20 years with and without the trade.
4. Try one or two alternatives that could matter and preview them too,
   for example:
   - the same fund on another venue, when the tax treatment differs;
   - buying with an amount instead of units, or in whole units;
   - for a large lump sum, `project` with the amount spread over 12 months;
   - for a sale with a short-term gain, waiting until the holding period
     passes (US) and what that would save.

## Answer

- Lead with the one or two facts that should decide it.
- Then the figures, side by side when comparing alternatives.
- Never tell the person to place the trade; they place it at their broker.
- Once they say they placed it, offer to record it: call `record_trade` with
  exactly the fields of the preview they agreed to (and the price and fees
  they actually got). Then say what was recorded. If the server is
  read-only, they record it in the app's Plan a trade screen instead.
- Write in the person's language. Not financial advice; say so once.

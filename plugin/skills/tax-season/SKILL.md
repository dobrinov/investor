---
name: tax-season
description: Prepare the capital-gains and investment-income part of the person's tax return from their Investor books — realised gains by treatment, sales whose venue is unknown, payouts of private holdings, dividends and tax withheld abroad, forms and deadline. Use when they ask about taxes, their annual return, "what do I declare", or a specific tax year.
---

# Tax season

The tax rules come from the draft pack for the person's country of tax
residency. Packs are not checked by a tax professional: say so, and treat
the result as a worksheet to take to an accountant, not a filing.

## Steps

1. Ask which tax year if it is not obvious (usually last year).
2. Call `realized_gains` for that year. It returns each sale with its
   treatment and the pack's summary: taxable net result, estimated tax,
   allowance used, exempt gains, and gains it could not classify.
3. Go through anything that needs the person's attention:
   - `venue_unknown`: the exchange is missing, so it is unclear whether the
     gain is exempt. Ask where it was sold and explain the difference.
   - `not_estimated`: taxed outside the estimate (for example a US
     short-term gain, or physical gold taxed as income). Explain why.
   - payouts of private holdings taxed as employment income: they are not in
     the capital gains; note the amount and the tax already withheld.
4. Call `declaration` for that year. When the pack describes the forms, it
   returns each part of the return (appendix, code, the form's own column
   names) filled from the books, including dividends with the tax withheld
   abroad, interest and the holdings at 31 December, plus the steps to file.
   Without it, use `journal` to collect dividends and withholding.
5. Mention the deadline and anything the pack notes say it does not model
   (for example share matching in the UK).

## Answer

This skill only reads. If a sale's venue was missing and the person tells
you where it was, that is fixed in the app's General ledger, not here.

A worksheet: for each part of the return, the figures to type in under the
form's column names; then the sales behind them (date, instrument, proceeds,
cost, gain, treatment), the estimated tax, the steps to file, and open
questions. Amounts are in the file's currency; say so if the return
needs another. Write in the person's language.

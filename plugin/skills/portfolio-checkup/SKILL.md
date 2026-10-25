---
name: portfolio-checkup
description: A monthly check-up of the person's Investor portfolio — net worth, goals, emergency fund, idle cash, concentration and stale prices — ending in at most three concrete things to do. Use when they ask how they are doing, for a review, a check-in, or "anything I should do?".
---

# Portfolio check-up

Read the person's books through the `investor` tools and give a
short, honest check-up. Every number comes from a tool; never estimate or
recompute what a tool already returns.

## Steps

1. Call `portfolio`. Note net worth, unrealised gain, cash per broker and
   currency, private holdings, and any holding without a price or whose
   price is more than a week old.
2. Call `plan` and `goals`. For each goal note the chance of reaching it and
   the monthly saving it needs; for the emergency fund, the months covered.
3. Call `journal` with a limit of about 30 to see what happened lately:
   deposits, trades, dividends, fees.
4. Look for, in this order:
   - an emergency fund below its target;
   - a goal with a low chance, and what monthly amount would fix it;
   - cash well above the emergency fund that is not assigned to a goal
     (money that is not working), but only if the plan has monthly investing;
   - one holding above half of the investments, or one currency dominating;
   - fees that are large next to the amounts traded;
   - stale or missing prices that make the figures unreliable.
5. If a move looks worth trying, test it with `project` (for example
   investing idle cash now against spreading it over 12 months) and quote
   the middle and bad outcomes from the result.

## Report

- Start with one sentence: how things stand overall.
- Then a small table or list of the figures that matter.
- End with at most three things to do, most useful first, each with the
  number that supports it. If nothing needs doing, say so.
- Write in the person's language (Bulgarian if the file's language is `bg`).
- Say plainly that this is not financial advice, once, at the end.

## Acting on it

The check-up itself changes nothing. If the person wants to act, offer the
matching write and make it only after they say yes: `update_bank_balance`
for a balance that changed, `set_price` for a stale price, `save_goal` or
`update_profile` for a new monthly amount. Say what changed afterwards;
`undo_last_change` takes it back.

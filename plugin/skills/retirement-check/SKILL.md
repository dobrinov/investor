---
name: retirement-check
description: Check whether the person's Investor plan gets them to their goals and retirement, and test "what if" moves — investing more each month, a lump sum, cost averaging, withdrawals, retiring earlier or later. Use for questions about retiring, reaching a goal, "am I on track", "how much should I invest", or comparing moves over the years.
---

# Retirement and what-if check

Use `plan`, `goals` and `project` from the `investor` tools. The
projection simulates many futures; always report a range, never one number.

## Steps

1. Call `plan`. If age, monthly investing, spending or retirement age are
   missing, say which ones matter and ask for them, or use the projection
   anyway and state the assumption.
2. Call `goals` for the chance of each goal and the monthly saving needed.
3. Call `project` with the plan as it is (base market). Note the middle,
   bad (1 in 10) and good (1 in 10) outcomes at retirement, and the share of
   simulated futures in which retirement withdrawals are fully paid.
4. Test the moves the person asked about, or the obvious ones if they asked
   "what should I change": a higher monthly amount, a lump sum against the
   same money spread over 12 months, retiring two years later. Compare them
   against the plan in the same call so the differences are fair.
5. If the result depends heavily on the market assumption, show bear and
   bull too.

## Answer

- One sentence: on track or not, and by how much.
- A table: plan versus each move, with middle, bad case and the share of
  futures that fully pay retirement.
- The smallest change that gets them on track, if they are not.
- If they want to keep it, offer to save it: `update_profile` for a new
  monthly amount or retirement age, `save_goal` for a goal, or `save_move` so
  the move shows in the app's Projections. Write only after they agree.
- Write in the person's language. Not financial advice; say so once.

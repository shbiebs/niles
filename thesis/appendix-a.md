# Appendix A. Niles for Non-Technical Readers: Metaphors and Analogies

This appendix retells the whole thesis without formulas. Every metaphor corresponds to a precise mechanism in the main text; a technical reader may treat it as an index of intuitions.

## A.1 Niles in Simpler Terms

Imagine a bank that keeps one perfect book of everything that ever happened, and thousands of sticky-note summaries of whatever people are currently asking about. The book is sacred and never edited. The sticky notes are cheap, disposable, and always rewritable from the book. This thesis is about proving that the sticky notes can never lie, working out how many are worth keeping and which ones, and inventing a way of writing instructions to the bank in which certain mistakes — losing money, mixing currencies, spending without permission — cannot be written down at all.

## A.2 The Great Vault Ledger

At the centre sits one book in a vault. Every deposit, withdrawal, transfer and fee is a line in it, written once, in order, in ink. Nobody — not the bank's president, not the computer itself — has an eraser. Everything else in the whole system is a *derived* convenience: balances, statements, reports are all just rereadings of the book.

And the book is not really about money. It is about *facts written down in order and never revised*. Money is the strictest customer it has, which is why it was chosen; the same book works for warehouse stock or medical supplies.

## A.3 Why the Main Book Can Never Be Erased

Each page ends with a special seal — a fingerprint computed from everything on that page *plus the previous page's fingerprint*. Change one old line and every seal after it stops matching.

One honest caveat, because it matters. The seal does not physically prevent anyone from rewriting a page; it makes rewriting *detectable* — but only by someone who wrote down a fingerprint earlier and bothers to check it now. A vault nobody audits is a vault with a very good lock and nobody holding the key. Mistakes, meanwhile, are fixed the way accountants have fixed them for five centuries: with a new correcting line, never with an eraser.

## A.4 The Two Calendars

The bank keeps two calendars on every line: *when it happened in the world* and *when we wrote it down*. If a payment from Monday only reaches the bank on Thursday, both truths are kept. That is how the bank can answer the auditor's favourite trick question: "Never mind what you know now — what did you *believe* on Wednesday?" With two calendars, that has one exact answer.

This is not a nicety the bank chose. European payments law dictates the relationship between the two calendars — the day a payment counts for interest cannot drift arbitrarily from the day the money actually moved. A bank with one clock cannot even state the rule it is required to follow.

## A.5 The Endless Questions

Customers and regulators never read the vault book directly; they ask questions. What's my balance? What can I actually spend right now? What did I hold on this date three years ago? What is our total exposure, broken down by product, currency, desk, legal entity and country? What did I spend this month? Is this transaction suspicious, right now, in the eighty milliseconds before the card machine times out?

There are thousands of such questions, and the list is *open* — the risk regulator explicitly requires banks to answer new groupings on demand, including during a crisis. You cannot pre-compute a list that has no end. A few questions are asked constantly; most are asked once in a blue moon. That lopsidedness is the economic fact the whole design exploits — and, as we will see in A.13, it is also the assumption most likely to be wrong in some banks, which is why the design measures it instead of trusting it.

## A.6 The Clerk's Notebook

For each kind of question, a clerk keeps a notebook of pre-computed answers. Ask your balance and the clerk does not reread ten years of the vault book — she looks at her notebook. The notebooks are fast, handy, and *entirely reconstructible*, because everything in them came from the book.

## A.7 Nudging, Not Recounting

When a new transaction lands, the clerk does not recompute your balance from scratch. She *nudges* it: balance was 100, a deposit of 5 arrived, write 105. Nudging is thousands of times cheaper than recounting, and the mathematics behind it guarantees that nudging always gives exactly what a full recount would.

## A.8 A Wall of Flow Charts

How does a clerk know which notebook entries a new transaction should nudge? Behind the clerks hangs a wall of flow charts — one per kind of question — showing how raw lines flow, step by step, into finished answers. New lines ride the arrows; each stop updates its intermediate note.

## A.9 Erasing to Make Room

Notebooks are small; the book is enormous. So clerks erase entries nobody has asked about lately — the dormant account, the stale report. Erasing a *notebook* is always safe, because the notebook never held anything original. This is the crucial asymmetry: the book may never be erased; the notebooks may always be.

## A.10 Reading Backwards to Fill a Gap

When someone finally asks about an erased entry, the clerk walks the flow chart *backwards* — from the question, back through the intermediate stops, to the exact pages of the vault book that matter — recomputes just that one answer, and writes it back.

The thesis's first big theorem says the answer produced this way is *identical* to the answer of a clerk who never erased anything. Its corollary is the sentence the whole bank rests on: **erasing and reconstructing can never create or destroy money.**

There is a quieter reward too. In a bank whose main book could still be *edited*, a clerk reading backwards might catch the book mid-change and produce nonsense — and systems that work that way need an elaborate protocol of warnings and hand-offs to prevent five different kinds of accident. When the book cannot change, those five accidents cannot happen, and the whole protocol simply disappears. A great deal of machinery is avoided by making one thing impossible.

## A.11 Stamping Every Answer with a Moment

Every answer comes with a stamp: "correct as of page 41,209." Never a bare number — always a number *and its moment*. Arguments about "but my screen said something different" dissolve, because the two screens carried two different stamps and both were exactly right for their stamp.

## A.12 How Sure Do You Want to Be?

Different questions deserve different freshness. A management dashboard may run a few pages behind. A fraud check wants the most recent page it can get. A card authorization must see *this very second*, no matter the cost.

So the system offers a ladder of promises, from "at most a few pages stale, and never more than five seconds old" up to "absolutely current, guaranteed," and every notebook declares which rung it stands on. The second big theorem is about the price list of that ladder — and it proves the top rung is sometimes so expensive that erasing stops paying at all, and draws the map of exactly where.

## A.13 The Clerk Who Decides What to Keep

Someone has to decide which notebooks are kept complete, which are filled in on demand, which live in the filing cabinet instead of the desk, and which are not kept at all. Guessing is how banks do it today.

Instead there is a head clerk with a rule book. She watches how often each question is asked, how expensive it is to reconstruct an answer, *how long the reconstruction takes*, and how fresh the answer has to be — and she moves notebooks between the desk, the cabinet and the bin accordingly. Her rule book comes from a century of mathematics about renting versus buying: if you have paid to reconstruct something often enough that you could have kept it all along, keep it.

Two honest notes. First, the timing matters more than it looks: while a clerk is walking back to the vault, other people queue up asking the same question, and a rule that ignores that queue is provably the wrong rule. Second, the head clerk can never change what an answer *is*. She only changes where it comes from. That is a theorem, and it is what makes it safe to let her change her mind while the bank is open.

## A.14 Money You Have Versus Money You Can Spend

When you tap your card, the shop asks the bank to set money aside before anything has actually moved. That set-aside is a *hold*, and it is why two perfectly correct balances can disagree: the money is still in the account, and it is already spoken for.

In this design a hold is not a scribble in a margin — it is its own line in the vault book, and resolving it (charged, cancelled, expired) is another line. So "the money you have" and "the money you can spend" are simply two notebooks reading the same book with different rules. Regulators have written formal guidance about the harm caused when banks get the gap between those two numbers wrong; here the gap has a name, a declared freshness promise, and a test.

There is a hard limit worth stating. Making sure two people cannot spend the same money at the same moment genuinely requires the clerks to talk to each other — three separate branches of computer science prove that this particular kind of check cannot be made free. Simply moving money without any such check *can* be made free. So the design confines the expensive part to the one place that needs it (the moment of authorization) and lets everything else run cheap.

## A.15 Many Clerks and No Collisions

Hundreds of clerks read the book at once and never trip over each other, for one simple reason: sealed pages never change. You cannot collide over something nobody can move. The only place requiring turn-taking is the single desk where the current page is being written.

## A.16 Where the Notebooks Sit on the Desk

Hot notebooks stay open on the desk; cool ones go to the shelf; the book's old volumes retire to deep archive — yet every seal still verifies when a volume comes back up. Moving a volume between rooms never changes what it says, because identity lives in the fingerprints, not the shelf.

## A.17 Showing Your Work

Every answer can be traced back to the exact lines of the vault book that produced it, and re-derived from scratch to check. If a wrong line is later corrected, the bank can ask which answers it touched. This is not a logging system bolted on the side; it falls out of the same mathematics that computes the answers, which is why it can be afforded at all.

## A.18 The Order Form the Clerks Fill In

Niles, the language, is the bank's order form, and its trick is that the form itself refuses nonsense. A transfer that doesn't balance: the form won't accept it. Adding euros to dollars: the boxes physically don't line up. Spending past a limit without a manager's countersignature: there is no field for it. Whole categories of financial accident become *unwritable*, rejected before the instruction ever reaches a clerk.

One thing the form cannot do, and the thesis says so: it cannot promise you never go overdrawn. It can promise nobody *forgot to ask*. The asking still has to happen at the counter.

## A.19 The Four Beliefs Underneath It All

Everything rests on four beliefs. The flow of events never ends. Any "state" is just the running total of its events — keep the events and you can always rebuild the totals, but not the other way round. Tools should be built for how the world actually is, not for a convenient fiction of it. And therefore the right shape is one unerasable book plus freely erasable notebooks.

The second belief has a nice proof and a nicer consequence. Two very different histories — "someone deposited five pounds" and "someone deposited seven then withdrew two" — leave the same total. The total cannot tell you which happened; the history can always tell you the total. That is the whole argument for keeping the history.

## A.20 Coins in Separate Jars

Every currency is its own jar of coins, counted separately, conserved separately. An exchange is never coins teleporting between jars — it is two simultaneous, individually balanced moves, one in each jar, linked by a single order. So even mid-crash, mid-erasure, mid-anything, no jar's total is ever wrong.

## A.21 The Sealed Room and the Only Key

Some things on the forms — names, private memos — go into sealed envelopes before they ever reach the bank, and only the customer holds the key. The bank files, orders and fingerprints the envelopes without opening them.

The honest fine print: the bank cannot compute with what it cannot read. So each field declares upfront whether it is sealed, semi-sealed or open, and the order form enforces the declaration. There is a clever middle option for amounts — a kind of sealed envelope you can still *add up* without opening — which lets the bank check that everything balances while never seeing the figures.

## A.22 Beyond Banking

The bank is only the sternest customer, chosen because it tolerates nothing. The same machinery counts anything that must never silently appear or vanish: warehouse stock, carbon credits, airline miles, game gold. Money is the proving ground, not the boundary.

## A.23 What We Have Not Done Yet

Honesty belongs in the simple version too. The mathematics is written and checked by hand. The vault, the clerks and the head clerk are designed in detail, and one piece is genuinely built: the tiny, deliberately stupid program that defines what "correct" means — the one that answers every question by rereading the entire book from the beginning, slowly, so that the fast system can be checked against it. What has *not* happened is the measuring. No speed or memory number in this thesis was produced by running this system, and none is claimed. What is written down instead is exactly what will be measured, what each result is predicted to be, and — importantly — what result would prove the whole idea wrong.

## A.24 Why All This Matters

Today, institutions run on many machines that each hold a slightly different version of the truth, stitched together with duct tape and hope; every mismatch is a support ticket, an incident, or a fine. This thesis is an argument, with proofs, that one truth plus cheap disposable summaries is not just cleaner — it can be *guaranteed* correct, *priced* honestly, *decided* automatically, and *written down* in a language that refuses the worst mistakes. The vault book never lies, the sticky notes can always be rebuilt, and now there is mathematics saying exactly when that bargain is a good one — and when it isn't.

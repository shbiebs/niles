# Appendix A. Niles for Non-Technical Readers: Metaphors and Analogies

This appendix retells the whole thesis without formulas. Every metaphor corresponds to a precise mechanism in the main text; a technical reader may treat it as an index of intuitions. Where the main text has since qualified, narrowed or refuted something, this appendix says so in the same plain language — an approachable retelling that quietly kept an old claim alive would be the most misleading document in the thesis.

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

There are thousands of such questions, and the list is *open* — the risk regulator explicitly requires banks to answer new groupings on demand, including during a crisis. You cannot pre-compute a list that has no end. A few questions are asked constantly; most are asked once in a blue moon.

That lopsidedness is real, and for years it was this design's stated reason for existing: keep the popular answers, throw the rest away, and the more lopsided the bank, the better the bargain. **The measurement said otherwise, and this is the most important correction in the appendix.** When questions cluster hard onto a few accounts, the clerk who keeps *everything* also shrinks — the busy accounts are few, so her complete set of notebooks is small too. Keeping only the popular ones still wins, but by less and less as the lopsidedness grows. The advantage did not grow with skew; it shrank, from about twelve-to-one down to under three-to-one across the range that was tested.

So the honest version is this. Erasing pays when paper is expensive relative to the cost of reading backwards, and it pays best at a *middling* number of notebooks rather than at either extreme. The lopsidedness of the questions decides the *shape* of that trade — where the sweet spot sits and how sharp it is — not which way it points. A.15 is where that distinction does its work.

## A.6 The Clerk's Notebook

For each kind of question, a clerk keeps a notebook of pre-computed answers. Ask your balance and the clerk does not reread ten years of the vault book — she looks at her notebook. The notebooks are fast, handy, and *entirely reconstructible*, because everything in them came from the book.

## A.7 Nudging, Not Recounting

When a new transaction lands, the clerk does not recompute your balance from scratch. She *nudges* it: balance was 100, a deposit of 5 arrived, write 105. Nudging is thousands of times cheaper than recounting.

The mathematics guarantees that nudging gives exactly what a full recount would — for the kind of question this thesis actually proves it about. That kind is broad and it is the kind banks ask constantly: running totals kept per account, per currency, per desk. It does not yet cover questions that *compare* two books against each other, or that ask for a maximum rather than a total. Those are stated as an open case, honestly labelled, and not quietly folded in.

## A.8 A Wall of Flow Charts

How does a clerk know which notebook entries a new transaction should nudge? Behind the clerks hangs a wall of flow charts — one per kind of question — showing how raw lines flow, step by step, into finished answers. New lines ride the arrows; each stop updates its intermediate note.

## A.9 Erasing to Make Room

Notebooks are small; the book is enormous. So clerks erase entries nobody has asked about lately — the dormant account, the stale report. Erasing a *notebook* is always safe, because the notebook never held anything original. This is the crucial asymmetry: the book may never be erased; the notebooks may always be.

There is a distinction worth keeping, because the whole design turns on it. A blank line in a notebook can mean two quite different things: *nobody has ever asked this* and *someone asked, we worked it out, and we later rubbed it out to save room*. The second kind of blank remembers which page of the book it was last correct at. Confusing the two is how a system ends up answering "zero" to a question whose true answer it merely forgot.

## A.10 Reading Backwards to Fill a Gap

When someone finally asks about an erased entry, the clerk walks the flow chart *backwards* — from the question, back through the intermediate stops, to the exact pages of the vault book that matter — recomputes just that one answer, and writes it back.

The thesis's first big theorem says the answer produced this way is *identical* to the answer of a clerk who never erased anything. Its corollary is the sentence the whole bank rests on: **erasing and reconstructing can never create or destroy money.** As in A.7, this is proved for running totals kept per key, which is what the engine actually accepts today; comparisons between books and maxima are an open case with a stated shape, not a theorem.

There is a quieter reward too. In a bank whose main book could still be *edited*, a clerk reading backwards might catch the book mid-change and produce nonsense — and systems that work that way need an elaborate protocol of warnings and hand-offs to prevent five different kinds of accident. When the book cannot change, those five accidents cannot happen. The thesis argues that the whole protocol therefore disappears; that is a prediction of the design, and the experiment that would confirm or refute it by building the same thing both ways has not been run. It is named, along with what would count as failing it.

## A.11 Stamping Every Answer with a Moment

Every answer comes with a stamp: "correct as of page 41,209." Never a bare number — always a number *and its moment*. Arguments about "but my screen said something different" dissolve, because the two screens carried two different stamps and both were exactly right for their stamp.

The stamp is really two dates, not one, and this is subtler than it looks. A notebook entry says: *worked out from the book as of page 41,209, and still correct up to page 41,240, because nothing between those two pages touched this account.* So the clerk may serve that entry to anyone asking about any page in that window — and she stamps the answer with **the page the customer asked about**, not the page she happened to do the arithmetic on. Stamping it with her own working page would be a small, plausible lie: it would tell a customer asking about Tuesday that the answer is from Monday, and a system that then compared two such stamps would conclude, wrongly, that time had gone backwards.

## A.12 Two Customers at the Same Empty Page

Two people ask about the same erased entry at the same instant. One clerk is already walking to the vault. Should the second wait for her, or make the trip herself?

The answer is exact and slightly surprising: she may wait **only if she wants the answer as of the very same page**. If the first clerk went to compute Tuesday's figure and the second needs Thursday's, they are not doing the same work — they are folding different stretches of the book — and the second must go herself. Sharing the trip because the *question* matched, while the *moment* did not, would hand the second customer an answer stamped with a page it was never computed for. This is the same lie as in A.11, arrived at from the other direction, and it is why the rule is written as an exact match rather than a convenient one.

## A.13 Bookmarks Every Hundred Lines

Reading backwards from a question to the beginning of a ten-year book would be ruinous, and if that were the real cost the whole design would collapse: the price of an answer would grow with the bank's age for ever.

So the clerk leaves a bookmark. Every so often — say every hundred entries touching an account — she writes in the margin: *as of here, this account stands at this figure*. Reading backwards then means finding the nearest bookmark and folding only the handful of lines since, not the whole history. On average that is about half the gap between bookmarks, plus one.

This is the piece of the design that experiment *produced* rather than confirmed. The pricing law the thesis proves for its ladder of promises is simply false without bookmarks; the measurement that refuted it without them is what found the mechanism that makes it true. It is recorded as a contribution earned that way, and the honest note travels with it: the numbers reported for the full daemon were gathered with bookmarking switched off, which is exactly the configuration in which the law is not claimed to hold.

## A.14 How Sure Do You Want to Be?

Different questions deserve different freshness. A management dashboard may run a few pages behind. A fraud check wants the most recent page it can get. A card authorization must see *this very second*, no matter the cost.

So the system offers a ladder of promises, and every notebook declares in writing which rung it stands on. From loosest to strictest:

- **At most a little behind.** Never more than so many pages, and never older than so many seconds — whichever bites first. Good enough for a dashboard.
- **Never going backwards.** Within one conversation, successive answers never step back in time. A balance that reads 105, then 100, then 105 again is the complaint this rung exists to forbid.
- **You always see your own writes.** You just made a deposit; every answer you are given afterwards includes it.
- **One single moment.** Everything in one report — every account, every currency, every notebook consulted — comes from the same page of the book. No composite picture assembled from three different Tuesdays.
- **A single serial order.** Reads and writes together can be arranged in one queue that explains everything that happened.
- **This very second.** Anchored at the newest visible page, reflecting every write that finished before the question was asked, with any money set aside in the same indivisible instant.

Two things make this ladder honest. First, *correctness is not one of the rungs*. Every rung, including the loosest, gives an answer exactly equal to what a clerk who never erased anything would have given; the rungs differ only in **which moment** you are told about, never in whether the arithmetic is right. That is what the theorem in A.10 buys, and it is why erasing is safe at every level.

Second, the price list. The second big theorem says that at the strictest rung a clerk who finds a blank notebook has no choice but to walk to the vault while the customer waits — she cannot serve something slightly old to cover the trip, because "slightly old" is precisely what that rung forbids. So at the top of the ladder, erasing eventually stops paying for itself. That much is proved, and a real crossover was found and measured. What the thesis explicitly does **not** have is a map of exactly where the crossover sits: the formula that would place it contains a constant nobody has pinned down, and a map drawn with that constant guessed at 1 rather than 10 would be wrong by a factor of ten in the very region that matters. An earlier draft claimed the map. That claim is withdrawn, and what replaces it is a prediction registered in writing before the measurement was taken, which is the more honest instrument.

## A.15 The Clerk Who Decides What to Keep

Someone has to decide which notebooks are kept complete, which are filled in on demand, which live in the filing cabinet instead of the desk, and which are not kept at all. Guessing is how banks do it today.

The thesis describes a head clerk with a rule book. She would watch how often each question is asked, how expensive it is to reconstruct an answer, *how long the reconstruction takes*, and how fresh the answer has to be, and move notebooks between the desk, the cabinet and the bin accordingly.

**This head clerk does not exist.** She is designed in detail and she has not been built. What is in the building today is a clerk who keeps whatever was asked for most recently, another who weighs cost against delay, and fifty-two lines of scaffolding nobody has finished. An earlier draft of this thesis claimed her rule book came with a mathematical guarantee of near-optimality borrowed from the literature on renting versus buying. That guarantee was claimed for a rule that was never implemented, and it has been withdrawn — from her name, from the contribution, and from here.

Two things about her *are* established, and they are worth keeping. First, the timing genuinely matters: while a clerk is walking to the vault, other people queue up asking the same question, and a rule that ignores that queue is provably the wrong rule. Second — and this is a theorem, not a hope — she can never change what an answer *is*. She only changes where it comes from. That is what makes it safe, in principle, to let her change her mind while the bank is open. It is also why her absence costs correctness nothing and costs only efficiency.

## A.16 Money You Have Versus Money You Can Spend

When you tap your card, the shop asks the bank to set money aside before anything has actually moved. That set-aside is a *hold*, and it is why two perfectly correct balances can disagree: the money is still in the account, and it is already spoken for.

In this design a hold is not a scribble in a margin — it is its own line in the vault book, and resolving it (charged, cancelled, expired) is another line. So "the money you have" and "the money you can spend" are simply two notebooks reading the same book with different rules. Regulators have written formal guidance about the harm caused when banks get the gap between those two numbers wrong; here the gap has a name, a declared freshness promise, and a test.

There is a hard limit worth stating. Making sure two people cannot spend the same money at the same moment genuinely requires the clerks to talk to each other — three separate branches of computer science prove that this particular kind of check cannot be made free. Simply *moving* money without any such check can be made free. So the design confines the expensive part to the one place that needs it, the moment of authorization, and lets everything else run cheap. This is an old idea in a new setting: give each till its own float, and the tills only have to talk when a float runs dry.

## A.17 Many Clerks and No Collisions

Hundreds of clerks can read the book at once and never trip over each other, for one simple reason: sealed pages never change. You cannot collide over something nobody can move. The only place requiring turn-taking is the single desk where the current page is being written.

That argument is sound about the *book*. It is worth being blunt about what has not been shown. The prototype that produced most of this thesis's numbers has exactly one clerk; the running daemon lets many customers connect but serves their questions strictly one at a time, behind a single door. So nothing here has demonstrated that hundreds of clerks working simultaneously actually go faster — and one measurement hints the opposite, with the daemon flat or falling from two workers to four where the conventional database rose. The claim that immutability removes whole categories of collision is an argument from the shape of the design, and a good one. The claim that the bank is therefore fast with many clerks is not yet a claim this thesis has earned, and §9 names the contention experiment as the most valuable thing left to run.

## A.18 Where the Notebooks Sit on the Desk

Hot notebooks stay open on the desk; cool ones go to the shelf; the book's old volumes retire to deep archive — yet every seal still verifies when a volume comes back up. Moving a volume between rooms never changes what it says, because identity lives in the fingerprints, not the shelf.

The desk and the bin are real. The shelf and the deep archive — the in-between places, where a notebook is neither open nor thrown away — are described in the design and are not built. They belong to the head clerk of A.15, and they are waiting on her.

## A.19 Showing Your Work

The design says every answer should be traceable back to the exact lines of the vault book that produced it, re-derivable from scratch to check, and that if a wrong line is later corrected the bank should be able to ask which answers it touched. The argument for affording this is genuine: it falls out of the same mathematics that computes the answers, rather than being a logging system bolted on the side.

**None of it is built.** The component that would do it was written, found to be doing less than it claimed, and deleted; the claim that went with it has been withdrawn rather than left standing with a hopeful note. What remains true today is narrower and still worth something: every answer carries the moment it is true at (A.11), the book itself is complete and sealed, and a slow reference reader (A.21) can recompute any answer from the beginning on demand. Tracing *which* answers a bad line touched, and explaining a particular figure step by step, are designs in this thesis and not capabilities of the artifact.

## A.20 The Order Form the Clerks Fill In

Niles, the language, is the bank's order form, and its trick is that the form itself refuses nonsense. A transfer that doesn't balance: the form won't accept it. Adding euros to dollars: the boxes physically don't line up. Spending past a limit without a manager's countersignature: there is no field for it. Whole categories of financial accident become *unwritable*, rejected before the instruction ever reaches a clerk.

Three honest limits, because "the form refuses nonsense" is the kind of sentence that grows in the retelling.

It cannot promise you never go overdrawn. It can promise nobody *forgot to ask*. The asking still has to happen at the counter — and even that promise is narrower than it was first written, with one known gap recorded openly rather than patched quietly.

It checks what it can see. The checker reasons within a stretch of instructions it can follow end to end, and it follows an instruction into another one only by copying it out in full. Where it cannot decide, it says **undecided** and hands the question on — which is the right behaviour, and is not the same thing as approving.

And the coverage that has been counted is small. Every conservation obligation across both codebases — sixteen of them — is discharged by the checker rather than left to the runtime. Sixteen obligations across four files, written by people who knew what the checker could prove, is real evidence and a modest amount of it. The separate claim that this checking costs nothing at run time has never been measured at all.

## A.21 The Slow Clerk Who Is Always Right

In the corner sits a clerk who has no notebooks. Every question, she answers by opening the vault book at page one and reading forward to the end. She is absurdly slow and completely incapable of being wrong, because there is nothing in her method that could drift.

She is the definition of correctness in this project, not a backup. The fast system is never checked against a list of expected answers written down by a person; it is checked against her, on randomly generated histories, including histories with crashes and erasures injected. Ten thousand transfers, five different random worlds, roughly three thousand erasures and three thousand historical questions per world: no disagreement, and every seal verified.

Two details make this more than a comforting anecdote. She is written independently of the fast engine, so a misunderstanding shared by both is less likely to hide. And the tests are run with deliberate sabotage as a control — withhold one page's worth of changes and check that the comparison *notices*. A test that cannot fail is not evidence, and an early version of one of these tests could not fail; it was found, and fixed, and the fix is what made the result mean anything.

## A.22 The Same Trick in a Rented Building

Here is the most uncomfortable finding in the thesis, and it belongs in the simple version.

Somebody built the whole scheme — notebooks, erasure, reading backwards, stamps — on top of an ordinary off-the-shelf database, with no new engine at all. It worked. It gave the same answers, with no disagreements.

So the case for building a new engine from scratch cannot be *"this is impossible otherwise."* It is not impossible otherwise. The case has to be made on what the purpose-built version adds: speed, guarantees held by construction rather than by convention, and mistakes the rented building permits and this one forbids. The thesis grades its own components on exactly this scale — does this part *enable* something otherwise impossible, or does it merely *add* to something already achievable? — and reports honestly which ones score which. Several score the weaker grade.

## A.23 What Is Actually New Here

A fair reader will ask what has not been done before, since demand-filled notebooks over a stream of changes are not a new idea; a well-known research system did that a decade ago.

The difference is what the notebooks are anchored to. In that earlier system there is no book — only the notebooks and the changes flowing into them — so a clerk reading backwards is reading a source that may be moving under her, and elaborate protocols exist to prevent the resulting accidents. Here the source is sealed and ordered, which is what makes an answer's *moment* a well-defined thing, what makes erase-and-rebuild provably harmless, what lets a promise about freshness be written on a notebook and checked, and what lets a language's guarantees about money survive contact with eviction. The theory is about what you can prove when the bottom of the stack cannot change; the engine exists to make those proofs testable.

There is also a plainer difference, recorded because it started this project. An attempt to use that earlier research system as a baseline failed, not on semantics but on availability: across a long session it was never built, never started and never connected to, and no query ran against it. That is an argument about artifacts, not about ideas, and it is labelled as such.

## A.24 Speaking the Old Language

A bank cannot rewrite twenty years of tooling to try a new idea. So the vault answers questions in the two languages the existing tools already speak: a real reporting tool connects to it over an ordinary connection and gets ordinary-looking answers back, byte for byte the same as the tool expects.

The discipline behind this matters more than the compatibility. The old language is accepted as a *way of speaking*, never as a second set of meanings: whatever arrives is translated immediately into the same internal form as everything else, checked by the same rules, and run by the same machinery. There is exactly one set of meanings in this system, and one moment of translation. A question that cannot be translated faithfully is refused loudly rather than answered approximately — refusing is recoverable, and quietly answering a slightly different question is not.

Half of this is built and driven by a real unmodified client. The other half — the second old language — has its translation written and tested but has no door on the building yet: nothing is listening on the other end.

## A.25 The Four Beliefs Underneath It All

Everything rests on four beliefs. The flow of events never ends. Any "state" is just the running total of its events — keep the events and you can always rebuild the totals, but not the other way round. Tools should be built for how the world actually is, not for a convenient fiction of it. And therefore the right shape is one unerasable book plus freely erasable notebooks.

The second belief has a nice proof and a nicer consequence. Two very different histories — "someone deposited five pounds" and "someone deposited seven then withdrew two" — leave the same total. The total cannot tell you which happened; the history can always tell you the total. That is the whole argument for keeping the history.

There is a measurement behind the fourth belief that is the strongest single number in this thesis, and it is worth stating plainly. As the book grows, the cost of answering a question from a warm notebook does not grow with it. The measured slope of answer-cost against book-length is indistinguishable from flat, while a conventional database's slope over the same data is clearly, unmistakably uphill. Keeping everything for ever, and paying nothing per answer for having kept it, is the bargain this design claims; that is the number that says the bargain is real.

## A.26 Coins in Separate Jars

Every currency is its own jar of coins, counted separately, conserved separately. An exchange is never coins teleporting between jars — it is two simultaneous, individually balanced moves, one in each jar, linked by a single order. So even mid-crash, mid-erasure, mid-anything, no jar's total is ever wrong.

## A.27 The Sealed Room and the Only Key

Some things on the forms — names, private memos — would go into sealed envelopes before they ever reach the bank, with only the customer holding the key. The bank would file, order and fingerprint the envelopes without opening them.

The honest fine print is twofold. First the design: the bank cannot compute with what it cannot read, so each field must declare upfront whether it is sealed, semi-sealed or open, and the order form enforces the declaration. There is a clever middle option for amounts — a kind of sealed envelope you can still *add up* without opening — which would let the bank check that everything balances while never seeing the figures.

Second, and more important: **this is a design, not a working feature.** The declarations exist in the language and are checked. The sealed-but-addable envelopes are not implemented. The ordinary transport-level encryption that protects a connection is deliberately delegated to established libraries rather than written here, which is the right call and worth saying out loud, because "we wrote our own cryptography" is the sentence that precedes most cryptographic disasters.

## A.28 The Compiler That Is Learning to Build Itself

The order form of A.20 has to be translated into something a machine can run, by a translator that is itself a program. Today that translator is written in an existing language. The intention is for Niles to describe its own translation — the traditional proof that a language is complete enough to be serious.

Two of the stages of that journey exist: a simple runner for the language, and the part that reads raw text and breaks it into words, written in Niles itself. The parts that understand grammar, check types and produce machine code are not yet written in Niles.

The reason this matters to a non-technical reader is a famous and genuinely unsettling one. If you build a translator with a translator, and the *old* translator has been tampered with, it can teach the new one to carry the same tampering — and the tampering will not appear anywhere in the text of either program. Worse, the usual test of success, in which a translator builds itself twice and produces exactly identical output, is also exactly what a successful attack of this kind looks like. The thesis states this limit rather than claiming the gate proves more than it does.

## A.29 Beyond Banking

The bank is only the sternest customer, chosen because it tolerates nothing. The same machinery should count anything that must never silently appear or vanish: warehouse stock, carbon credits, airline miles, game gold.

This one was actually put to the test rather than asserted, in the strongest form available: write a warehouse-stock program and see what breaks. Units of a product moving between warehouses conserve exactly as money does, the checker proves the conservation, the program runs and seals — and nothing in the translator, the internal form or the engine had to change, which a test now enforces by checking that none of them contains the word *warehouse* or *sku*.

And it found one real limit, which is recorded rather than fixed. The notation for a quantity insists that its unit be written as exactly three lowercase letters — the shape of an international currency code. So `15000 jpy` can be written down and `5 widget` cannot, and every quantity in the warehouse program has to be passed in from outside instead of written plainly. The machinery is general; the handwriting still assumes money. The one-line widening was deliberately not applied, so that the finding would stay visible instead of becoming a footnote about something that used to be true.

## A.30 When This Is the Wrong Answer

A design that is right for everything is a design that has not been examined. Seven cases where a bank should choose something else:

- **When everything is asked about equally often and everything must be exactly current.** Erasing pays for itself only when some answers are hotter than others. Flat demand plus maximum strictness everywhere is precisely the corner where this stops being a bargain — and it is not a hypothetical corner; published measurements of real caching workloads sit in it.
- **When the law requires real erasure of a primary fact.** The vault's whole virtue is that nothing can be removed. Destroying the key to an encrypted record gets *closer to the effects of* erasure without being erasure, and a regulator may reasonably decline the distinction. This thesis chooses immutability with its eyes open, and that choice has a cost.
- **When the work is enormous one-off scans.** A warehouse-sized analytical question that reads everything once, and will never be asked again, is better served by a system designed for reading everything once.
- **When writes must land in microseconds.** The book is sealed in batches, and a batch has a minimum tick. Trading systems that count in microseconds will refuse that floor.
- **When the parties do not trust each other.** Multiple independent operators who might actively cheat need a different kind of design altogether, which is deliberately out of scope here.
- **When the application is small or has no invariants to protect.** An ordinary database on ordinary software is the rational default, and most applications are that. The machinery in this thesis earns its cost only where getting it wrong is expensive.
- **When one single account must be authorized against thousands of times a second.** The coordination limit of A.16 is real and no systems trick removes it. The answer there is a business change — split the account, net the traffic — not a cleverer engine.

## A.31 What We Have Done and What We Have Not

Honesty belongs in the simple version too, and the shape of this paragraph has changed since it was first written.

**Built and running.** The vault book, with real durability and real crash recovery. The notebooks, with erasing, reading backwards and bookmarks. The slow reference clerk of A.21. The order form of A.20, from raw text through checking to execution. A door onto the building that a real, unmodified, widely-used reporting tool connects to and is answered by correctly. A separate, complete core-banking platform — accounts, holds, schedules, foreign exchange, lending, trade finance and the rest — built independently and checked by its own five hundred tests. A measuring instrument that counts every scrap of memory the engine asks for, so that a claim about memory is a counted number rather than an impression.

**Measured.** Real numbers now exist, and the sentence that used to stand here — that none did — was retired. Correctness against the reference clerk, across randomised worlds with crashes and erasures. The pricing of each rung of the ladder. The point where erasing stops paying, located. Wall-clock speed against a mature conventional database, on the same machine, on five different kinds of work. Memory, counted exactly. And the flatness result of A.25, which is the one that matters most.

**Refuted or withdrawn.** The belief that lopsided demand makes the bargain better: measured, false, restated (A.5). The claim to have drawn the map of where erasing stops paying: withdrawn (A.14). The near-optimality guarantee for the head clerk: withdrawn, and she is unbuilt (A.15). Tracing an answer's ancestry: withdrawn and deleted (A.19). Several speed comparisons come back at or below the conventional database, and are reported as such — three of five targets were missed, and one of the three that was met flipped to missed when the same test was run on a second machine of the same type, which says as much about the honesty of a single measurement as it does about the engine.

**Not done.** Many clerks working at once, which is the largest hole (A.17). The head clerk (A.15). The second old language's door (A.24). Tracing and impact analysis (A.19). The sealed envelopes (A.27). The self-built translator beyond its first two stages (A.28). Distribution across machines, and agreement between machines, are designed and not built.

The reason for listing all of this in the accessible appendix rather than burying it in a table is that a reader who only reads this appendix should come away with the same picture as a reader who reads all of it — including the parts where the answer was no.

## A.32 Why All This Matters

Today, institutions run on many machines that each hold a slightly different version of the truth, stitched together with duct tape and hope; every mismatch is a support ticket, an incident, or a fine. This thesis is an argument, with proofs, that one truth plus cheap disposable summaries can be *guaranteed* correct, *priced* honestly, and *written down* in a language that refuses the worst mistakes. Deciding what to keep automatically — the fourth thing an earlier draft of this sentence promised — is designed and not delivered, and is named here as future work rather than left in a list of accomplishments.

The vault book never lies. The sticky notes can always be rebuilt. There is now mathematics saying when that bargain is a good one and measurement saying, in several specific cases, that it is not. A thesis that could only report the cases where it won would be a worse thesis, and a less useful one to the next person who tries.

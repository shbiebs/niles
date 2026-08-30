# Two-Span Diagnostics for a Money-Safety Type System: Evidence, Design Practice, and Where the Claim Overreaches

## 0. Verdict up front

Your claim — *"a money-safety diagnostic must point at the violation and at the rule that forbids it"* — divides into two sub-claims with very different evidentiary status.

- **Sub-claim A (content):** a diagnostic should carry the violated *rule/warrant*, not just the symptom. **This is empirically supported**, by a controlled comparative study with 68 professional developers [1], [2].
- **Sub-claim B (presentation):** the rule must be delivered as a *second highlighted source span*. **This has no direct empirical support.** I found no controlled study comparing single-span against multi-span diagnostics, in any language, for any error class. The support for B is (i) the type-error-localization/slicing theory, which establishes that a static error is a *set* of program points [3]–[6], and (ii) industrial convergence across rustc, Clang, GCC and the Language Server Protocol [7]–[12]. That is a *design* argument, not an *empirical* one, and you should say so in the thesis.

Notably, the compiler most often held up as best-in-class for humane errors — Elm — is largely **single-region plus prose** [13], [14]. That is a real counter-datapoint you must address.

---

## 1. Empirical studies of compiler error message quality

### 1.1 Barik et al., ICSE 2017 — what it actually found

Your recollection is right, and the study is stronger than you remember [1].

Design: 56 undergraduate/graduate participants (mean 1.4 years professional experience), 10 compiler-error scenarios derived from Google's analysis of 26 million builds, defects injected into Apache Commons Collections, five minutes per task, Eclipse on Windows, GazePoint GP3 eye tracker, screen recording synchronised at 10 fps.

Findings:

1. **Developers do read error messages.** "Participants allocate a substantial portion of their total task to reading error messages (13%–25%)."
2. **Reading them is as hard as reading code.** Mean fixation duration on error messages 419 ms (SD 270, n = 18,573) versus 394 ms on source code (SD 240, n = 81,098), against ~275 ms for silent English reading in prior work. Kullback–Leibler divergence between code and error-message fixation distributions was 0.059 (negligible); both diverged sharply from silent reading (3.38 and 2.37).
3. **Reading difficulty predicts task success**, but weakly: χ² on revisits vs. correctness G² = 60.9, p < .0001, yet Nagelkerke R² = 0.16. The authors are explicit: "Reading difficulty is only one of many factors."
4. **Situational awareness matters.** In task T2 the compiler emitted two identical messages for two child classes; 55 of 56 participants wrongly concluded both needed the missing method, when the correct fix was in the parent. This is the closest thing in the empirical literature to evidence that *where* the message points changes the fix developers choose.
5. They explicitly recommend **integrated context presentation**, citing LLVM `scan-build`'s presentation of an error "as a sequence of steps that the developer can follow alongside in the context of the code."

Conclusion quoted verbatim: "error messages matter."

**Use for you:** point 4 and point 5 are your best empirical hooks. Point 4 is a *misattribution-of-blame* result, which is exactly the failure mode your two-span design targets. But it is a single task in a single study and was not designed to test span count.

### 1.2 Becker et al., ITiCSE-WGR 2019 — the landscape survey

The 12-author working-group report [15] is the canonical survey. Relevant conclusions:

- It confirms Barik's findings as pivotal (points 1–4 above are quoted in its Section 1.1).
- It names **"the locality problem: errors are frequently detected far away from their generation sites"** as a first-class technical challenge, with worked examples: a missing brace reported at end-of-file; C++ template errors reported *inside library code*; Scheme macro errors reported against the expanded form; and the observation that "type-inference rules can be complicated and non-local, often resulting in inscrutable error messages that are far from the source of the error."
- On enhancement it is candid: "evidence seems to be conflicting… but may not be as conflicting as it appears," citing Denny 2014 (null) against Becker 2016 (positive), and noting "much work has taken place since then with no clear consensus." It also concedes that "some message design guidelines… are also conflicting, leaving the way forward unclear."

**Use for you:** the locality problem is the survey's own framing of *why one span is not enough*. It stops short of prescribing multi-span, but it is the strongest survey-level statement you can cite.

### 1.3 Denny, Luxton-Reilly, Carpenter, ITiCSE 2014 — the negative result

You need this and you should present it fairly [16].

Design: 83 first-year students, six-week accelerated CS1, University of Auckland, Summer 2013; randomised into control (n = 42) and intervention (n = 41); ten Java exercises in the CodeWrite web tool. Enhanced messages recognised 53 error types in 9 categories and included the offending line, an explanation of the likely cause, "a table showing two code fragments side by side" (wrong vs. corrected), and a fix explanation.

Measures and results — all null:
- Longest run of consecutive non-compiling submissions: p ∈ [0.08, 0.89]
- Total non-compiling submissions: p = 0.9471
- Attempts to resolve three common errors: p ∈ [0.24, 0.44]

**Limits the authors themselves state**, which materially constrain how far the result generalises to you:
- *A confound in the wrong direction for the enhancement:* "raw compiler feedback shows up to two compiler errors, while the enhanced feedback module displays only one." The treatment condition showed **less** information, not more.
- Students may not have engaged with the verbose messages.
- The worked examples may not have transferred to students' own code.
- The authors recommend an observational study to see actual usage.

**Additional limits you should state:** the population is CS1 novices, the errors are *syntax* errors, the intervention is prose-and-example enrichment (not span structure), and the outcome measures are submission-count proxies rather than fix correctness or blame accuracy. Guzdial's critique is that the enhancement was not grounded in a model of student intent and that the title over-generalises [17].

### 1.4 The rest of the enhancement literature — genuinely mixed

- **Becker, SIGCSE 2016** [18], N in a CS1 cohort using Decaf as the sole compiler: found positive effects — but Becker himself attributes the divergence from Denny to *measurement philosophy* (Decaf captured all compilation activity; CodeWrite captured submissions) rather than to contradictory phenomena [19].
- **Pettit, Homer, Gee, SIGCSE 2017** [20]: "Results Inconclusive." Their tool (Athene) ran *after* local development, so most errors were never observed.
- **Prather et al., ICER 2017** [21]: the important paradox. Tested *in isolation*, enhanced messages outperformed standard ones. In a 60-minute think-aloud with 31 students, there was "no substantial increase in student learning outcomes over the control," yet qualitative analysis showed students *were* reading the enhanced messages and "generally make effective changes after encountering them." Their explanation is contextual cognitive load.
- **Denny, Prather, Becker et al., CHI 2021** [22]: builds a readability construct for error messages.
- **Santos, Becker et al., UKICER 2024** [23]: 106 students, six buggy C programs; GPT-4-generated messages beat conventional compiler messages in only 1 of 6 tasks, and **hand-crafted expert explanations beat both** on objective and subjective measures.

**Net reading:** message *enhancement* has a genuinely mixed record, and the mechanism that most reliably wins is expert-authored, task-specific content — not volume.

### 1.5 The one study that supports your content claim: Barik's Toulmin work

Barik, Ford, Murphy-Hill and Parnin, ESEC/FSE 2018 [2], and Barik's dissertation [24], model an error message as a *Toulmin argument*: **Claim** (the assertion of error), **Grounds** (evidence), **Warrant** (the rule licensing the inference from grounds to claim), **Backing** (support for the warrant), plus qualifier and rebuttal.

Method: a comparative evaluation with **68 professional developers**, plus an empirical analysis of compiler error messages and their Stack Overflow explanations across seven languages.

Findings, precisely:
- "Developers significantly prefer the error message that employs proper argument structure over a deficient argument structure **when neither offers a resolution** — but will accept a deficient argument structure **if it provides a resolution**."
- Human-authored Stack Overflow explanations converge on three shapes: messages offering a resolution, simple arguments, and extended arguments with additional evidence/backing.

**This is your best empirical citation.** Your `conserve per (txn, cur)` rule *is* the Warrant. Pointing at where it is declared *is* the Backing. But read the second half of the sentence: a resolution beats structure. See §4.

### 1.6 The gap: no study of multi-span vs. single-span

I searched for controlled experiments comparing diagnostics that highlight one location against diagnostics that highlight two or more (including eye-tracking work on which location developers attend to first). **I found none.** The eye-tracking literature in software engineering covers code reading and, in Barik et al., the *distribution of attention across IDE panes* — not across spans within a diagnostic.

State this in the thesis as an open empirical question. It is also, incidentally, a cheap and publishable experiment you could run as thesis validation: same violation, three renderings (single span; span + prose warrant; span + second span at the rule), measured on blame-attribution accuracy and time-to-correct-fix.

---

## 2. Design practice — primary sources

### 2.1 rustc

**Structure.** A rustc diagnostic has a level, an optional error code, a message ("It should be general and able to stand on its own, so that it can make sense even in isolation"), a diagnostic window, spans, and sub-diagnostics [7]. Spans have two orthogonal attributes — *primary* (rendered with `^^^`) and *labelled*. The dev guide's rule for primary spans: they "should have enough text to describe the problem in such a way that if it were the only thing being displayed (for example, in an IDE) it would still make sense." Note what this concedes: **the primary span must survive alone.** Secondary spans are, by rustc's own doctrine, additive context.

The API is literally a set of spans: `MultiSpan { primary_spans: Vec<Span>, span_labels: Vec<(Span, DiagMessage)> }` [8].

`help` vs `note` is normative: "`help` should be used to show changes the user can possibly make to fix the problem. `note` should be used for everything else."

**Applicability** [7], [9] — the four levels verbatim:
- `MachineApplicable` — "Can be applied mechanically."
- `HasPlaceholders` — "Cannot be applied mechanically because it has placeholder text in the suggestions."
- `MaybeIncorrect` — "Cannot be applied mechanically because the suggestion may or may not be a good one."
- `Unspecified` — "Cannot be applied mechanically because we don't know which of the above cases it falls into."

**Error codes and `--explain`.** RFC 1567 [10] normalises long explanations into five sections: error description, minimal example, error explanation (the "*why*"), how to fix, and optional additional information. This is architecturally important for you: **rustc's answer to "where is the rule?" is partly `--explain`, i.e. an out-of-band rule statement with no span at all.**

**The declarative two-span idiom.** The diagnostic-struct derive documentation gives this canonical example [9]:

```rust
#[primary_span]
#[label("field already declared")]
pub span: Span,
#[label("`{$field_name}` first declared here")]
pub prev_span: Span,
```

That is exactly your "violation + the thing it conflicts with" shape, in rustc's own docs.

### 2.2 rustc's borrow/move checker — concrete output (generated on rustc 1.95.0, edition 2021)

**Use after move (E0382) — your `hold consumed twice` case.** Source: a `Hold` struct passed by value twice.

```
error[E0382]: use of moved value: `hold`
 --> move.rs:8:21
  |
6 |     let hold = Hold { amount: 40 };
  |         ---- move occurs because `hold` has type `Hold`, which does not implement the `Copy` trait
7 |     let a = consume(hold);
  |                     ---- value moved here
8 |     let b = consume(hold);
  |                     ^^^^ value used here after move
  |
note: consider changing this parameter type in function `consume` to borrow instead if owning the value isn't necessary
 --> move.rs:3:15
  |
3 | fn consume(h: Hold) -> i64 { h.amount }
  |    -------    ^^^^ this parameter takes ownership of the value
  |    |
  |    in this function
note: if `Hold` implemented `Clone`, you could clone the value
 --> move.rs:1:1
  |
1 | struct Hold { amount: i64 }
  | ^^^^^^^^^^^ consider implementing `Clone` for this type
...
7 |     let a = consume(hold);
  |                     ---- you could clone this value
```

**Span count: seven labelled source locations across three diagnostic windows.** The main window is exactly your three-span shape — *bound here* (line 6), *first consumed here* (line 7), *consumed again here* (line 8, primary).

**Conflicting mutable borrows (E0499) — three spans:**

```
error[E0499]: cannot borrow `i` as mutable more than once at a time
 --> borrow.rs:4:13
  |
3 |     let x = &mut i;
  |             ------ first mutable borrow occurs here
4 |     let a = &mut i;
  |             ^^^^^^ second mutable borrow occurs here
5 |     *x += 1;
  |     ------- first borrow later used here
```

**E0502 — three spans**, same shape (immutable borrow here / mutable borrow here / immutable borrow later used here).

**The purest "violation + rule" instances.** Two rustc diagnostics point literally at the rule that forbids the code:

```
error[E0277]: `Money` doesn't implement `std::fmt::Display`
...
note: required by a bound in `show`
 --> bound.rs:2:12
  |
2 | fn show<T: Display>(t: T) { println!("{}", t); }
  |            ^^^^^^^ required by this bound in `show`
```

```
error: unused variable: `balance`
 --> lint.rs:3:9
  |
3 |     let balance = 40;
  |         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_balance`
  |
note: the lint level is defined here
 --> lint.rs:1:9
  |
1 | #![deny(unused_variables)]
  |         ^^^^^^^^^^^^^^^^
```

`note: required by a bound in ...` and `note: the lint level is defined here` are the industrial precedent for `the conserve per (txn, cur) rule declared here`. Cite these two verbatim; they are the single strongest design-practice evidence you have.

### 2.3 Clang

Clang's "Expressive Diagnostics" page [11] documents: full column information and caret ("The point… exactly shows where the problem is, even inside of a string"); **range highlighting** ("captures and accurately tracks range information for expressions, statements, and other constructs… highlights the left and right side of the plus which makes it immediately obvious what the compiler is talking about"); typedef preservation with "aka"; Fix-it hints "for fixing small, localized problems"; template type diffing; automatic macro expansion with nested range information; and notes pointing to candidate declarations elsewhere. Note that Clang's *ranges* are largely intra-expression; its cross-location mechanism is the `note:`.

### 2.4 GCC — Malcolm's labelled ranges

David Malcolm's 2018 patch adding labelled source ranges [12] states the goal as clarifying "where two aspects of the source aren't in sync," and gives:

```
pr69554-1.c:11:18: error: invalid operands to binary +
11 |   return (p + 1) + (q + 1);
   |          ~~~~~~~ ^ ~~~~~~~
   |             |         |
   |             |         const char *
   |             const char *
```

and, for a parameter mismatch, an error at the call site plus `note: initializing argument 2` at the *declaration*. Malcolm's own framing: the dual-location strategy makes mismatches clear "without requiring users to cross-reference distant code locations." That sentence is worth quoting — it is a design rationale for exactly your claim, from a compiler maintainer.

### 2.5 Elm — and the counter-datapoint

Elm's stated philosophy, "compilers should be assistants, not adversaries" [13], is endorsed and quoted in WG21 P2429R0 [25]. Elm messages explain *why*, in plain English, with a suggestion, and treat errors as teaching opportunities [14].

**But**: the Elm error examples I could retrieve — including the type-mismatch cases in the official error-message-catalog issue tracker [26] — highlight **one source region**, with the second party to the conflict expressed as *prose type text* rather than a second span. I was unable to retrieve the body of "Compiler Errors for Humans" (the page is a single-page app), so treat my Elm characterisation as sourced from the catalog and secondary quotations, not from that article's text.

This matters: Elm is the most-cited exemplar of good compiler diagnostics and it is essentially single-span. Your thesis must not claim industrial consensus on multi-span; the honest claim is *consensus on multi-*location*, delivered variously as secondary spans (rustc), notes at declarations (Clang, GCC, rustc), or prose (Elm).*

### 2.6 LSP — the machine-readable convergence

The Language Server Protocol's `Diagnostic` carries [27]:

```
/**
 * An array of related diagnostic information, e.g. when symbol-names within
 * a scope collide all definitions can be marked via this property.
 */
relatedInformation?: DiagnosticRelatedInformation[];
```

with `DiagnosticRelatedInformation { location, message }` — "used to point to code locations that cause or are related to a diagnostics."

This is the strongest form of the industrial-convergence argument: the *protocol* every modern IDE speaks has a first-class slot for "the other location," and it was put there because compilers needed it.

---

## 3. Type-error localization and slicing — your strongest theoretical support

This is where your claim is genuinely well-founded, and you should lead with it.

**Wand, POPL 1986** [3] opened the problem of *finding the source* of a type error rather than reporting the unification failure point.

**Haack & Wells** [4], [5] are the key citation. Their thesis, verbatim: "We present a new approach that identifies the location of a type error as a **set of program points (a slice)** all of which are necessary for the type error." Their critique of algorithms W, M and UAE: they "often fail to identify the real location of the error. They identify **one node** of the program tree which participates in the type error, but will often be the wrong node to blame."

They define two properties and prove them:
- **Completeness/faithfulness (Thm 6.1):** the slice preserves the error.
- **Accuracy/minimality (Thm 6.2):** removing any point from a minimal error slice makes the program well-typed.

They also observe that a slice "correctly includes all of the parts of the program where changes can be made to fix the type error" and "correctly excludes all of the parts of the program where changes can not fix the type error" — and that a standalone minimal-program rendering is "especially useful if relevant program points are far apart, possibly in multiple files."

**Zhang & Myers, POPL 2014** [6] generalise: errors are unsatisfiable constraints; the method analyses satisfiable *and* unsatisfiable constraints to identify the expressions most likely responsible, ranking explanations Bayesianly "under the assumption that the programmer's code is mostly correct, so the simplest error explanations are chosen." Instantiated for OCaml type inference and Jif information flow, it "identifies the location of program errors significantly more accurately than do existing compilers."

**Chen & Erwig, POPL 2014** [28] attack the same thing from the repair side: "one of the problems of the approach taken by GHC is that it **commits to a single error location**, because in some cases the program text does not contain enough information to confidently make the right decision about the correct error location." They generate "all (program-structure-preserving) type changes that can possibly fix the type error," ranked and presented iteratively, and note that debugging type errors "seems to be an inherently ambiguous undertaking."

**The precise argument this licenses for you:** if the violation is constituted by a *set* of constraints, then a diagnostic that names one program point is reporting an arbitrarily-chosen member of that set, and the choice is a heuristic, not a fact about the program. For a `conserve per (txn, cur)` violation the set is at minimum {the postings that fail to net, the rule declaration}; for double-consumption it is {binding, first consume, second consume}. That is a *soundness-of-blame* argument and it does not depend on any human-subjects result.

**The precise limit:** none of [3]–[6], [28] evaluates with human subjects whether *showing* the set beats showing one member. They evaluate localization accuracy against known-correct ground truth. The theory says the error *is* a set; it does not say the UI must render the set.

---

## 4. Steelman: the case against you

Assemble it from the sources above; it is stronger than it looks.

1. **Resolution beats structure.** Barik et al.'s own result: developers prefer proper argument structure only *when neither message offers a resolution*; given a resolution, they accept a deficient argument [2]. If a `MachineApplicable` fix exists ("add a balancing posting of 40.00 USD to `cash:ops`"), the second span may be dead weight. This is the strongest single counter-argument and it comes from the same paper that supports you.

2. **Enhancement has a mixed-to-null record.** Denny et al. found no effect on any of three measures [16]. Pettit et al. were inconclusive [20]. Prather et al. found messages that demonstrably worked *in isolation* produced no learning gain *in situ*, and attributed this to cognitive load [21]. Santos et al. found richer LLM-generated messages beat baseline in 1 of 6 tasks [23]. A second span is enhancement; the prior on enhancement working is not high.

3. **Reading cost is real and measured.** Error-message reading is *at least as effortful per fixation as source-code reading* (419 ms vs 394 ms) and already consumes 13–25% of task time [1]. Every additional span is additional reading in the most expensive modality, and Barik et al. specifically identify the *hybrid* nature of messages ("developers must context-switch between two different modalities of reading") as a cost driver. A second span in a different file is worse still: it is a navigation, not a glance.

4. **rustc's own doctrine undercuts you.** The dev guide requires that the *primary* span alone "would still make sense" in an IDE [7], and instructs authors to "reduce the span to the smallest amount possible." Secondary spans are explicitly context, not load-bearing. Furthermore, rustc's canonical mechanism for delivering the *rule* is the error code plus `--explain` [10] — out of band, no span.

5. **The best-loved compiler is single-region.** Elm's celebrated diagnostics highlight one region and put the counterparty in prose [26], and its verbosity is *already* criticised as paternalistic and terminal-hostile [14], [26].

6. **Practitioners ask for less, not more.** The Elm catalog contains a standing request for a compact mode so errors fit in a small terminal [26]; the same pressure exists in every large codebase where one edit produces fifty diagnostics.

7. **Rule-declaration spans may be uninformative in practice.** If `conserve per (txn, cur)` is declared once, in a prelude, at the top of a schema, then the second span points at the same three lines every time. Repeated identical context is precisely the pattern that trains people to stop reading — and Barik et al.'s T2 result shows that repeated identical messages actively misled 55 of 56 participants [1].

**How to answer it (fairly, not dismissively):** concede 1 and 6 outright; make the rule-span *conditional* — suppressed when the rule is lexically obvious or has been shown recently, and always subordinate to a `MachineApplicable` suggestion where one exists. Answer 7 by making the second span point at the *narrowest* rule text and by carrying the rule's identity in the message even when the span is elided.

---

## 5. Your specific error class: linear/affine and effect diagnostics

**There is essentially no research literature on diagnostics for linear or affine type systems, or for effect systems, as such.** I searched for it and found system papers (Idris 2 / quantitative type theory [29]; Granule [30]) that specify the type systems but do not study error reporting, and no HCI or empirical work on how such errors should be presented. This is a genuine gap and you should claim it as one — it is a contribution opportunity, not a weakness, provided you don't paper over it.

The industrial case is rustc, and the evidence is in §2.2. Concretely:

| Error | Distinct labelled locations | Structure |
|---|---|---|
| E0382 use-after-move | 7 (across 3 windows) | binding / first move / use-after-move (primary) + type-decl notes |
| E0499 two mutable borrows | 3 | first borrow / second borrow (primary) / first borrow later used |
| E0502 mut vs. immut borrow | 3 | immutable borrow / mutable borrow (primary) / immutable borrow later used |
| E0277 unsatisfied bound | 4 | call / argument (primary) + type decl + **`required by this bound in show`** |
| `deny` lint violation | 2 | violation (primary) + **`the lint level is defined here`** |

Your three cases map cleanly:
- *"this hold is consumed twice" / "first consumed here" / "bound here"* — **this is E0382, verbatim in structure.** rustc puts the primary caret on the *second* use, not the first. Follow that: the primary span is the violation the user must edit.
- *"does not conserve USD; net movement is −40.00" + "the `conserve per (txn, cur)` rule declared here"* — **this is the lint-level / trait-bound pattern.** Two spans, primary at the transaction, `note:` at the rule.
- *"promises `ledger_consistent` but reads at `read_your_writes`" + contract + offending read* — three spans, mirroring E0502's shape (declaration / conflict / the use that makes it matter).

On the usability side: Crichton et al.'s formative study (N = 36 Rust learners, Ownership Inventory across dangling pointers, overlapping borrows, illegal borrow promotion, lifetime parameters) found participants were **64% accurate at predicting *why* the compiler rejected code but only 31% accurate at explaining the resulting undefined behaviour** [31] — i.e. borrow-checker messages successfully convey the *rule* but not the *stakes*. Crichton's earlier position paper argues error messages should be "used pedagogically, to teach the programmer how to avoid a class of type errors in the future" [32]. For a money-safety system, the analogue of "stakes" is *conservation*: your message should say what would have been lost, which is what "net movement is −40.00" already does. That number is doing more work than the second span.

---

## 6. Corrections — where you are overclaiming

1. **You have no direct empirical evidence for multi-span over single-span.** None exists, for any error class. Do not write "studies show." Write: *there is no direct empirical evidence; the support is (a) type-error-slicing theory, (b) industrial convergence, and (c) an indirect empirical result about argument structure.*

2. **Barik et al. ICSE 2017 does not support multi-span.** It supports "developers read messages," "reading is hard," and "reading difficulty predicts performance (R² = 0.16)." Its T2 misattribution finding is suggestive, not evidential. Don't stretch it.

3. **Barik et al. FSE 2018 supports *content*, not *spans*.** A Warrant can be prose. "The `conserve per (txn, cur)` rule requires every transaction to net zero in each currency" satisfies Toulmin without any second span. Your claim needs an extra premise — that a *span* is a better vehicle for the warrant than *prose* — and that premise is untested.

4. **Denny et al. 2014 cuts against you less than it appears, but you must not overstate the rebuttal.** The legitimate points are: syntax errors, CS1 novices, submission-count proxies, and the authors' own note that the enhanced condition showed *one* error where the control showed *two*. The illegitimate move is to dismiss it; it is a properly randomised controlled study and its result stands for what it tested.

5. **"Industrial convergence" is on multi-*location*, not multi-*span*.** rustc, Clang, GCC and LSP converge on "point at the other place." Elm does not; it uses prose. And rustc's principal vehicle for the *rule* is the error code plus `--explain`, which has no span at all. Phrase the claim as convergence on *dual reference*, with span-vs-note-vs-prose as an open rendering question.

6. **rustc's primary-span doctrine is a constraint on your design, not a licence.** "Primary spans should have enough text to describe the problem in such a way that if it were the only thing being displayed… it would still make sense" [7]. If your money-safety error is *unintelligible* without the second span, rustc's own guidance says the primary label is underspecified. Design so the second span is *sufficient*, never *necessary*.

7. **There is no literature on linear/affine or effect-system diagnostics.** Claim the gap; do not cite phantom work.

8. **You cannot yet claim that developers attend to the second span.** No eye-tracking study has looked inside a diagnostic. If this claim matters to the thesis, run the experiment — it is small, it is well-scoped, and it would be the first of its kind.

**Defensible reformulation:**

> A money-safety violation is constituted by a set of program points, not a single one — the postings that fail to conserve, and the rule that declares conservation. Following the type-error-slicing tradition [4]–[6], [28], reporting a single point reports an arbitrary member of that set. We therefore render such diagnostics with a primary span at the transaction and a secondary reference to the governing rule, following the precedent of rustc's `required by this bound in …` and `the lint level is defined here` and the LSP `relatedInformation` mechanism. We note that no controlled study has compared multi-span against single-span diagnostics; the indirect empirical support is Barik et al.'s finding that developers prefer messages with proper argument structure (claim + grounds + warrant), qualified by their finding that a resolution outweighs structure — which is why every Niles money-safety diagnostic carries a suggestion at a declared `Applicability` level, and the rule reference is suppressed when a machine-applicable fix is available.

That last clause turns your weakest point into a design decision.

---

## References

[1] T. Barik, J. Smith, K. Lubick, E. Holmes, J. Feng, E. Murphy-Hill, and C. Parnin, "Do developers read compiler error messages?," in *Proc. 39th Int. Conf. Software Engineering (ICSE)*, 2017, pp. 575–585. doi: 10.1109/ICSE.2017.59. https://doi.org/10.1109/ICSE.2017.59 · PDF: https://jssmith1.github.io/assets/pdf/ICSE_2017_EYE.pdf

[2] T. Barik, D. Ford, E. Murphy-Hill, and C. Parnin, "How should compilers explain problems to developers?," in *Proc. 26th ACM Joint Meeting European Software Engineering Conf. and Symp. Foundations of Software Engineering (ESEC/FSE)*, 2018, pp. 633–643. doi: 10.1145/3236024.3236040. https://doi.org/10.1145/3236024.3236040

[3] M. Wand, "Finding the source of type errors," in *Proc. 13th ACM SIGACT-SIGPLAN Symp. Principles of Programming Languages (POPL '86)*, 1986, pp. 38–43. doi: 10.1145/512644.512648. https://doi.org/10.1145/512644.512648

[4] C. Haack and J. B. Wells, "Type error slicing in implicitly typed higher-order languages," in *Programming Languages and Systems (ESOP 2003)*, LNCS 2618, 2003, pp. 284–301. doi: 10.1007/3-540-36575-3_20. https://doi.org/10.1007/3-540-36575-3_20 · PDF: http://www.macs.hw.ac.uk/~jbw/papers/Haack+Wells:Type-Error-Slicing-in-Implicitly-Typed-Higher-Order-Languages:ESOP-2003.pdf

[5] C. Haack and J. B. Wells, "Type error slicing in implicitly typed higher-order languages," *Science of Computer Programming*, vol. 50, no. 1–3, pp. 189–224, 2004. doi: 10.1016/j.scico.2004.01.004. https://doi.org/10.1016/j.scico.2004.01.004

[6] D. Zhang and A. C. Myers, "Toward general diagnosis of static errors," in *Proc. 41st ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 2014, pp. 569–581. doi: 10.1145/2535838.2535870. https://doi.org/10.1145/2535838.2535870 · Project page: https://www.cs.cornell.edu/andru/papers/diagnostic/

[7] The Rust Project, "Errors and lints," *Rust Compiler Development Guide*. https://rustc-dev-guide.rust-lang.org/diagnostics.html

[8] The Rust Project, "Struct `rustc_errors::MultiSpan`," *rustc API documentation*. https://doc.rust-lang.org/beta/nightly-rustc/rustc_errors/struct.MultiSpan.html

[9] The Rust Project, "Diagnostic structs," *Rust Compiler Development Guide*. https://rustc-dev-guide.rust-lang.org/diagnostics/diagnostic-structs.html

[10] G. Couprie et al., "RFC 1567: Long error codes explanation normalization," *The Rust RFC Book*, 2016. https://rust-lang.github.io/rfcs/1567-long-error-codes-explanation-normalization.html

[11] LLVM Project, "Clang: Expressive Diagnostics." https://clang.llvm.org/diagnostics.html

[12] D. Malcolm, "[committed] diagnostics: add labeling of source ranges," gcc-patches mailing list, Aug. 2018. https://gcc.gnu.org/pipermail/gcc-patches/2018-August/504640.html · GCC option documentation: https://gcc.gnu.org/onlinedocs/gcc/Diagnostic-Message-Formatting-Options.html

[13] E. Czaplicki, "Compiler Errors for Humans," elm-lang.org, 2015. https://elm-lang.org/news/compiler-errors-for-humans · "Compilers as Assistants," 2015. https://elm-lang.org/news/compilers-as-assistants *(Note: both pages are client-rendered; the body text could not be retrieved programmatically for this review. Claims about Elm here are sourced from [14], [25], [26].)*

[14] "Elm — Amazing, Informative, Paternalistic Error Messages," Jamalambda's Blog, Jun. 2021. https://jamalambda.com/posts/2021-06-13-elm-errors.html

[15] B. A. Becker, P. Denny, R. Pettit, D. Bouchard, D. J. Bouvier, B. Harrington, A. Kamil, A. Karkare, C. McDonald, P.-M. Osera, J. L. Pearce, and J. Prather, "Compiler error messages considered unhelpful: The landscape of text-based programming error message research," in *Proc. Working Group Reports on Innovation and Technology in Computer Science Education (ITiCSE-WGR)*, 2019, pp. 177–210. doi: 10.1145/3344429.3372508. https://doi.org/10.1145/3344429.3372508 · PDF: https://amirkamil.com/papers/iticse19.pdf

[16] P. Denny, A. Luxton-Reilly, and D. Carpenter, "Enhancing syntax error messages appears ineffectual," in *Proc. 2014 Conf. Innovation & Technology in Computer Science Education (ITiCSE '14)*, 2014, pp. 273–278. doi: 10.1145/2591708.2591748. https://doi.org/10.1145/2591708.2591748 · Slides: https://www.cs.auckland.ac.nz/courses/compsci747s2c/lectures/errors.pdf

[17] M. Guzdial, "Enhancing syntax error messages appears ineffectual — if you enhance the error messages poorly," *Computing Ed Research*, Jul. 2014. https://computinged.wordpress.com/2014/07/29/enhancing-syntax-error-messages-appears-ineffectual-if-you-enhance-the-error-messages-poorly/

[18] B. A. Becker, "An effective approach to enhancing compiler error messages," in *Proc. 47th ACM Technical Symp. Computing Science Education (SIGCSE)*, 2016, pp. 126–131. doi: 10.1145/2839509.2844584. https://doi.org/10.1145/2839509.2844584

[19] B. A. Becker, "The enhancing compiler error messages saga: the saga continues," *CS0*, Mar. 2017. https://cszero.wordpress.com/2017/03/31/the-enhancing-compiler-error-messages-saga-the-saga-continues/

[20] R. S. Pettit, J. Homer, and R. Gee, "Do enhanced compiler error messages help students? Results inconclusive.," in *Proc. 2017 ACM SIGCSE Technical Symp. Computer Science Education*, 2017, pp. 465–470. doi: 10.1145/3017680.3017768. https://doi.org/10.1145/3017680.3017768

[21] J. Prather, R. Pettit, K. H. McMurry, A. Peters, J. Homer, N. Simone, and M. Cohen, "On novices' interaction with compiler error messages: A human factors approach," in *Proc. 2017 ACM Conf. Int. Computing Education Research (ICER)*, 2017, pp. 74–82. doi: 10.1145/3105726.3106169. https://doi.org/10.1145/3105726.3106169

[22] P. Denny, J. Prather, B. A. Becker, C. Mooney, J. Homer, Z. C. Albrecht, and G. B. Powell, "On designing programming error messages for novices: Readability and its constituent factors," in *Proc. 2021 CHI Conf. Human Factors in Computing Systems*, 2021, pp. 1–15. doi: 10.1145/3411764.3445696. https://doi.org/10.1145/3411764.3445696

[23] E. A. Santos and B. A. Becker, "Not the silver bullet: LLM-enhanced programming error messages are ineffective in practice," in *Proc. 2024 Conf. United Kingdom & Ireland Computing Education Research (UKICER)*, 2024. doi: 10.1145/3689535.3689554. https://doi.org/10.1145/3689535.3689554 · arXiv: https://arxiv.org/abs/2409.18661

[24] T. Barik, "Error messages as rational reconstructions," Ph.D. dissertation, North Carolina State Univ., 2018. https://static.barik.net/barik/thesis/barik-thesis.pdf

[25] S. Brand, "Concepts error messages for humans," ISO/IEC JTC1/SC22/WG21 document P2429R0, 2021. https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2022/p2429r0.pdf

[26] The Elm Project, "error-message-catalog" and issue #41, "Compact error messages." https://github.com/elm/error-message-catalog · https://github.com/elm/error-message-catalog/issues/41

[27] Microsoft, "Language Server Protocol Specification — 3.17: `Diagnostic`, `DiagnosticRelatedInformation`." https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/

[28] S. Chen and M. Erwig, "Counter-factual typing for debugging type errors," in *Proc. 41st ACM SIGPLAN-SIGACT Symp. Principles of Programming Languages (POPL)*, 2014, pp. 583–594. doi: 10.1145/2535838.2535863. https://doi.org/10.1145/2535838.2535863 · PDF: https://web.engr.oregonstate.edu/~erwig/papers/CF-Typing_POPL14.pdf

[29] E. Brady, "Idris 2: Quantitative type theory in practice," in *Proc. 35th European Conf. Object-Oriented Programming (ECOOP)*, LIPIcs vol. 194, 2021, art. 9. https://drops.dagstuhl.de/storage/00lipics/lipics-vol194-ecoop2021/LIPIcs.ECOOP.2021.9/LIPIcs.ECOOP.2021.9.pdf

[30] D. Marshall, M. Vollmer, and D. Orchard, "Linearity and uniqueness: An entente cordiale," in *Programming Languages and Systems (ESOP 2022)*, LNCS 13240, pp. 346–375. doi: 10.1007/978-3-030-99336-8_13. https://granule-project.github.io/papers/esop22-paper.pdf

[31] W. Crichton, G. Gray, and S. Krishnamurthi, "A grounded conceptual model for ownership types in Rust," *Proc. ACM Program. Lang.*, vol. 7, no. OOPSLA2, art. 265, 2023. doi: 10.1145/3622841. https://doi.org/10.1145/3622841 · arXiv: https://arxiv.org/abs/2309.04134

[32] W. Crichton, "The usability of ownership," in *Proc. Human Aspects of Types and Reasoning Assistants (HATRA '20)*, 2020. arXiv: https://arxiv.org/abs/2011.06171

[33] A. J. Kennedy, "Programming languages and dimensions," Ph.D. dissertation, Univ. of Cambridge, Tech. Rep. UCAM-CL-TR-391, 1996. https://www.cl.cam.ac.uk/techreports/UCAM-CL-TR-391.pdf *(relevant to the currency-mismatch case; dimension/unit type systems, not diagnostics)*

[34] V. J. Traver, "On compiler error messages: What they say and what they mean," *Advances in Human-Computer Interaction*, vol. 2010, art. 602570, 2010. doi: 10.1155/2010/602570. https://doi.org/10.1155/2010/602570 *(cited as one of the four guideline sources consolidated in [15]; full text not retrieved for this review)*

---

**Provenance note on the rustc outputs in §2.2:** all diagnostics were generated locally with `rustc 1.95.0 (59807616e 2026-04-14)`, `--edition 2021 --color=never`. Sources are in `/tmp/claude-0/-home-claude/51bd9638-671b-5c48-b805-80af69862a9e/scratchpad/diag/` (`move.rs`, `borrow.rs`, `borrow2.rs`, `bound.rs`, `lint.rs`, `mismatch.rs`) and are trivially reproducible — worth including as a reproducible artifact in an appendix rather than citing blog transcriptions.

**Two things I could not verify and you should not assert:** (a) the body text of Czaplicki's "Compiler Errors for Humans"; (b) Traver's specific enumerated principles. Both should be read directly before citing detail from them.

# Reading the Code

**A plain-language guide to Niles, to Rust, and to the Nilestream and GBS engines — for people who have never read a line of code in their lives.**

---

## Who this is for, and what it promises

This guide assumes nothing. Not that you know what a variable is, not that you know what a computer does when you press a key, not that you have ever seen a program. It is written for the person who has been handed a file full of symbols and told that it runs a bank.

It makes three promises.

**Every symbol is explained.** Not the important ones, not a representative sample — every keyword, every punctuation mark, every kind of literal value, every structure. If you meet something in one of these files and it is not in this guide, that is a defect in this guide.

**Everything is explained twice.** Once technically, in the terms a programmer would use, so that you are learning the real thing and not a children's version of it. And once by analogy, so that the technical explanation has something to stick to. The analogies are chosen to be *load-bearing*: where an analogy breaks down, this guide says where, because an analogy you trust past its breaking point is worse than none.

**It says what things are for.** A reference that tells you `mut` means "mutable" has told you nothing. The question worth answering is why a language would bother to have a word for that, what goes wrong without it, and what the person who typed it was worried about.

This document is **not part of the thesis**. Appendix A of the thesis retells the *ideas* without formulas. This retells the *code* without assuming a reader. They overlap deliberately in places, and where they do, this one goes further into the machinery.

## How to use it

There are six parts, and they are meant to be read in order the first time.

**Part I** is about computers, not about any language. What a processor does, and what it is made of — true and false, the gates that combine them, and the transistors that make the gates. What "running a program" means. What an abstraction is — which is the single most important idea in the whole document. Skip this part only if you already know it.

**Part II** is Niles, the language this project invented.

**Part III** is Rust, the language the engines are actually written in. It is a full reference, not a summary: Rust is where most of the code in these two repositories lives, and a guide that covered the invented language thoroughly and the real one loosely would have things backwards.

**Part IV** is the vocabulary of the engines themselves — the invented words that are not part of either language but that you will meet on every page of this particular project.

**Part V** is the complete lookup: every keyword of both languages, every sign, every literal form, every type, alphabetically, each with its plain meaning and its metaphor. This is the part you come back to.

**Part VI** reads two real files line by line, one in each language.

A convention: text in `this typeface` is literal code — something that appears in a file exactly as written. When a word is being discussed as a word, it appears like `mut`. When a concept is being discussed, it appears in ordinary type.

---

# Part I — Before any language

## 1. What a processor actually does

Strip away everything and a computer's processor does one thing: it repeatedly fetches a number from memory, interprets that number as an instruction, and carries it out. Then it does it again. Billions of times a second, for as long as it is powered.

The instructions are astonishingly simple. Genuinely, the whole repertoire is of this kind:

- Copy the number at location 4,192 into slot A.
- Add slot A to slot B, put the answer in slot A.
- If slot A is zero, jump to instruction 8,800; otherwise carry on.
- Write slot A to location 4,196.

That is close to all of it. There is no instruction for "calculate a balance", no instruction for "sort these names", certainly no instruction for "check that this transfer conserves money". There is add, subtract, compare, copy, and jump-if.

**The analogy.** Imagine an unimaginably fast clerk who is also unimaginably literal. He sits at a desk with a small number of scratch pads (those are the *registers* — the "slots" above) and behind him is a warehouse of numbered pigeonholes stretching out of sight (that is *memory*). He works from a list of orders, one at a time, in order. Each order is one of: fetch the contents of pigeonhole 4,192 onto scratch pad A; add pad A to pad B; if pad A is zero, skip ahead to order 8,800; put pad A into pigeonhole 4,196.

He is not clever. He cannot be asked to "handle the payroll". He cannot notice that an order is obviously a mistake. He will carry out a nonsensical order at full speed for a thousand years without complaint. Everything a computer appears to do — every image, every conversation, every bank — is built out of billions of those four kinds of order per second.

Three consequences follow, and they explain most of what the rest of this guide is about.

**The processor has no idea what anything means.** A number in a pigeonhole might be a price, a letter of the alphabet, a colour, or the address of another pigeonhole. The number itself carries no label. Meaning is entirely in what the program *does* with it. This is why languages have *types* (§6): the meaning has to live somewhere, and if it does not live in the machine it has to live in the language.

**The processor has no idea what is allowed.** There is no instruction for "this account may not go negative". If money must be conserved, something above the processor has to arrange it. The whole design of Niles is an answer to "where should *something above* live, and how early can it check?"

**Everything is a number, including the orders.** The list of orders is itself in the pigeonholes. This is why a program can write a program — and it is why Part I §5, on how text becomes machine instructions, is a story about a program transforming other programs.

## 2. Underneath arithmetic: true, false, and the switch

§1 said the processor's whole repertoire is add, compare, copy and jump. This section is about what those are made of, because they are not primitive either. They are built from a single, much simpler idea — a switch that is either on or off — and the mathematics of switches is small enough to fit in a few pages and worth knowing in full, because the same mathematics reappears, unchanged, in every `if`, every `where`, every `&&` you will read in Part II and Part III.

### 2.1 Two values

Start with statements that are either true or false. *The account is overdrawn.* *This posting is in dollars.* *The card has expired.* There is no third option, and no "somewhat". Call true **1** and false **0**.

Why insist on exactly two? Because two is the number of positions a switch has, and a switch is the cheapest, most reliable thing that can be manufactured by the billion. Everything that follows is a consequence of that manufacturing fact. A computer is not binary because binary is elegant; it is binary because on-and-off is the only distinction that can be made ten billion times a second with no mistakes.

### 2.2 Three operations

Given true-or-false values, there are exactly three basic things worth doing with them. Each is defined by a **truth table** — a complete list of every input and what comes out, which is the whole definition, with nothing left to interpretation.

**AND** — true only when both inputs are true.

| a | b | a AND b |
|---|---|---|
| 0 | 0 | 0 |
| 0 | 1 | 0 |
| 1 | 0 | 0 |
| 1 | 1 | 1 |

*The picture:* a safe with two keyholes that must be turned together. Either key alone does nothing.

**OR** — true when at least one input is true.

| a | b | a OR b |
|---|---|---|
| 0 | 0 | 0 |
| 0 | 1 | 1 |
| 1 | 0 | 1 |
| 1 | 1 | 1 |

*The picture:* two doorbells wired to one chime. Press either, or both, and it rings. Note that this is the inclusive "or": both is fine. Everyday English often means the exclusive kind — "soup or salad" — and that one has its own name below.

**NOT** — flips the one input.

| a | NOT a |
|---|---|
| 0 | 1 |
| 1 | 0 |

*The picture:* the denial of a statement. "The account is overdrawn" becomes "the account is not overdrawn".

In the code you will read, these three are written `&&`, `||`, `!` in Rust, and in Niles either those or the words `and`, `or`, `not` — the two spellings are the same operator and produce the same result, which a test in this repository checks.

### 2.3 The laws: boolean algebra

The three operations obey a small set of laws, which together are called **boolean algebra** after George Boole, who wrote them down in 1847 as a way of doing logic by calculation. Every one can be checked by writing out the truth table for both sides and seeing that they match — there is nothing to take on trust.

Some of them look like ordinary arithmetic, with AND behaving like multiplication and OR like addition:

- **Commutative:** `a AND b = b AND a`, and the same for OR. Order does not matter.
- **Associative:** `(a AND b) AND c = a AND (b AND c)`, and the same for OR. Grouping does not matter.
- **Distributive:** `a AND (b OR c) = (a AND b) OR (a AND c)`. Just like multiplying out brackets.
- **Identity:** `a AND 1 = a`; `a OR 0 = a`. Anding with true, or oring with false, changes nothing.
- **Annihilation:** `a AND 0 = 0`; `a OR 1 = 1`. Anding with false is always false; oring with true is always true.

And some of them look nothing like arithmetic, which is where it becomes its own subject:

- **Idempotence:** `a AND a = a`; `a OR a = a`. Asking the same question twice does not change the answer. (In arithmetic, `x + x` is not `x`.)
- **The other distributive law:** `a OR (b AND c) = (a OR b) AND (a OR c)`. This one has no arithmetic counterpart at all — you cannot distribute addition over multiplication — and it holds here.
- **Complement:** `a AND (NOT a) = 0`; `a OR (NOT a) = 1`. A statement and its denial are never both true, and one of them is always true.
- **Double negation:** `NOT (NOT a) = a`.
- **Absorption:** `a OR (a AND b) = a`; `a AND (a OR b) = a`. If the safe opens with key A alone, then "key A, or keys A and B together" is just key A.

And the two that are most useful in practice, **De Morgan's laws**:

- `NOT (a AND b) = (NOT a) OR (NOT b)`
- `NOT (a OR b) = (NOT a) AND (NOT b)`

*The picture:* two doorbells and one chime. "The chime did not ring" means "nobody pressed A *and* nobody pressed B". The denial of an OR is an AND of denials. Similarly, "the safe did not open" means "key A was not turned *or* key B was not turned" — the denial of an AND is an OR of denials.

Why a reader of this repository should care about a list of laws: they are the reason a compiler can rewrite `!(a && b)` as `!a || !b` and know the program means the same thing; they are how a database decides which order to test the conditions in a `where` clause; and they are how a chip designer takes a circuit of a thousand switches and finds an equivalent one with six hundred. Every optimisation of a condition, in software or in silicon, is an application of these dozen lines.

### 2.4 The derived operations, and the one that builds all the rest

A few combinations come up so often they have names.

**XOR** ("exclusive or") — true when the inputs *differ*. This is the everyday "soup or salad": one or the other, not both. Written `^` in both languages when applied to numbers.

| a | b | a XOR b |
|---|---|---|
| 0 | 0 | 0 |
| 0 | 1 | 1 |
| 1 | 0 | 1 |
| 1 | 1 | 0 |

**NAND** ("not and") — AND, then flipped. True unless both inputs are true.

**NOR** ("not or") — OR, then flipped. True only when both inputs are false.

Here is the fact that makes computers buildable. **Every operation on true-and-false values, however complicated, can be built from NAND alone.** Not from AND, OR and NOT — from just NAND. The demonstration is three lines:

- `NOT a` = `a NAND a`. (Feed the same input to both sides. If `a` is 1, NAND gives 0; if 0, it gives 1.)
- `a AND b` = `NOT (a NAND b)` = `(a NAND b) NAND (a NAND b)`.
- `a OR b` = `(NOT a) NAND (NOT b)` — which is De Morgan's law, read backwards.

With NOT, AND and OR in hand, everything else follows. So a factory that can make one kind of component well, and connect copies of it together, can make anything. NOR has the same property. This is called *functional completeness*, and it is why the next section only has to explain how to build one gate.

### 2.5 The transistor is a switch

A **transistor** is a switch with no moving parts. It has three connections. Current can flow between two of them — call them *in* and *out* — but only when a voltage is applied to the third, the **gate**. Apply the voltage and the path conducts; remove it and the path is broken. The gate controls the path without touching it, the way a small valve controls a large pipe.

*The picture:* a tap. The gate is the handle; the water is the current; the handle can be turned by a trickle of water from *another* tap. That last part is what makes it more than a tap — a switch that can be thrown by the output of another switch is a switch that can be arranged into a chain of reasoning.

Two flavours are manufactured, and the pairing between them is what modern chips are made of. An **n-type** transistor conducts when its gate is *high* (voltage on). A **p-type** transistor conducts when its gate is *low* (voltage off). The two are mirror images, and a design that uses both is called **CMOS** — the technology behind essentially every processor made since the 1980s.

*Where the picture breaks.* A tap wears out; a transistor switches ten billion times a second for twenty years. A tap is either open or shut; a transistor has an in-between region that designers work hard to pass through as fast as possible, because time spent there wastes power as heat. And a real transistor leaks a little even when off — which is why a chip with billions of them warms up doing nothing.

### 2.6 A gate from four transistors

Take a supply wire held high (call it 1) and a ground wire held low (0). The **NOT gate** — an *inverter* — is two transistors:

- A p-type between the supply and the output. It conducts when the input is *low*.
- An n-type between the output and ground. It conducts when the input is *high*.

Both transistors' control terminals are wired to the same input. Input high: the n-type conducts and pulls the output down to 0; the p-type is off. Input low: the p-type conducts and pulls the output up to 1; the n-type is off. Exactly one of the two paths is ever open, which is the NOT truth table, and is also why the gate draws almost no current while it is sitting still — there is never a path straight from supply to ground.

**NAND is four transistors**, and its shape is the whole secret:

- Two p-types *in parallel* between the supply and the output. If *either* input is low, one of them conducts and the output is pulled high.
- Two n-types *in series* between the output and ground. Only if *both* inputs are high do both conduct and the output get pulled low.

Read that again with §2.2 beside it. Switches in series are AND: the path exists only if both are closed. Switches in parallel are OR: the path exists if either is closed. The NAND gate is those two facts, physically, with the p-types handling the "either input low" case and the n-types handling the "both inputs high" case. Nothing is being computed in any mysterious sense; wires are being connected or not.

(NOR is the same four transistors with the arrangements swapped: p-types in series, n-types in parallel.)

### 2.7 From gates to the four orders of §1

Now the adder. Adding two single binary digits has four cases: 0+0=0, 0+1=1, 1+0=1, and 1+1=10 — that is, 0 with a carry of 1. Look at the two output columns:

- The *sum* digit is 1 exactly when the inputs differ. That is XOR.
- The *carry* digit is 1 exactly when both inputs are 1. That is AND.

So a **half adder** is one XOR gate and one AND gate. A **full adder** also accepts a carry coming in from the digit to its right — it is two half adders and an OR. Chain sixty-four full adders, each passing its carry to the next, and you have a machine that adds two 64-bit numbers. That is the `add` instruction of §1. (A chain that passes the carry along one digit at a time is slow, because the last digit must wait for all the others; real processors use cleverer arrangements that compute the carries in parallel, but they compute the same function.)

**Compare** is subtraction with the answer thrown away and only its sign kept. **Jump-if-zero** needs to know whether every one of sixty-four bits is 0: that is one enormous NOR. **Copy** is not a gate at all — it is wiring, and a clock pulse that says "now".

So all four of §1's orders are boolean logic, and boolean logic is switches, and switches are transistors. The tower of abstractions from §4 does not start at the processor; it starts at the truth table.

### 2.8 Remembering: a bit of memory from two gates

Everything so far computes an answer and forgets it. A register — the scratch pad of §1 — has to *hold* a value. The trick is feedback: take two NOR gates and connect each one's output to the other's input. The arrangement has two stable states, and once nudged into one it stays there, because each gate is holding the other in place. That is a **latch**, and it stores one bit for as long as the power is on.

A 64-bit register is sixty-four of them. A processor's **clock** is a steady pulse that says, billions of times a second, "everybody take your inputs now" — so that all the latches update together, in step, and the machine advances one order at a time rather than dissolving into a race.

### 2.9 Numbers as bits

A **bit** is one true-or-false, one latch, one wire. Numbers are written in bits the way decimal numbers are written in digits, except each position is worth twice the one to its right instead of ten times: `1011` is 8+0+2+1 = 11. Eight bits together make a **byte**, and can express 256 different values — which is why §22's `u8` runs from 0 to 255, and why adding 1 to 255 has nowhere to go. Sixty-four bits express about eighteen quintillion values; a hundred and twenty-eight bits, which is what this engine uses for money, express about 3.4 × 10³⁸ — roughly the number of atoms in a small asteroid, and some hundred billion billion times the grains of sand on Earth. That width is chosen so that a sum of every transaction a bank will ever process cannot overflow; it is *not* what makes the arithmetic exact. Exactness comes from holding money as whole minor units (§11), and where a genuine fraction arises — an exchange rate, an interest accrual — the rounding is a declared step, not a silent one.

Negative numbers use a convention called **two's complement** — flip every bit and add one — chosen because it lets the same adder circuit handle subtraction with no extra hardware. That is the kind of decision this whole section has been about: a cheap trick in the mathematics becomes a smaller circuit becomes a faster chip.

### 2.10 What this has to do with the code you will read

The connection is not decorative. It is the same logic at every level.

**`bool` is the type of one truth table cell.** In both languages it holds exactly `true` or `false`; every comparison — `==`, `<`, `>=` — produces one, and every `if`, `while` and `where` consumes one. An `if` on a comparison is the jump-if of §1 — and the specific case of "jump if zero" is the giant NOR of §2.7.

**`&&`, `||`, `!` are the three gates, and `and`, `or`, `not` are the same three.** With one addition that a gate does not have: **short-circuiting**. When a program evaluates `a && b`, it evaluates `a` first, and if `a` is false it does not evaluate `b` at all — the annihilation law says the answer is already 0. The same for `a || b` when `a` is true. This matters more than it looks: `x != 0 && total / x > 10` is safe *only because* the division is skipped when `x` is zero. The keyword table's entries for `and` and `or` say "short-circuiting" for exactly this reason, and it is the reason the operands' *order* is part of a program's meaning even though the commutative law says AND does not care about order. The law is about values; the program is about steps.

**`&`, `|`, `^` on integers are the gates applied side by side.** `12 & 10` ands the bits of 12 (`1100`) with the bits of 10 (`1010`) position by position and gets `1000`, which is 8. These sit at their own levels in Appendix B.10.1's precedence table — binding tighter than comparison and looser than arithmetic, which is Rust's ordering — because they are arithmetic on bits, not logic on truths, even though the operation inside each bit is the same one. (It is worth knowing that C orders them the other way, *below* comparison, so that in C `a == b | c` means `(a == b) | c`, a famous trap. Niles follows Rust, and a test confirms it parses `a == (b | c)`. Appendix B.10.1's own sentence describing this as "C's ordering rather than Python's" has it backwards on both counts; the table above that sentence is right, and the table is what the parser implements.)

**De Morgan's laws are why two spellings of one condition are interchangeable**, and why a compiler may quietly choose either. `!(overdrawn && !authorized)` and `!overdrawn || authorized` are the same program, and a reviewer who knows the law can read the second where the author wrote the first.

**A `match` is a truth table with the rows written out.** The exhaustiveness rule of §27 — every case must have an arm — is the requirement that the table have no blank rows. And the exhaustiveness *check* is boolean algebra run by the compiler over the set of patterns.

**The type-checker of §5 is a boolean function over the program.** "Does this program type-check?" is a single true-or-false answer, computed by combining thousands of smaller true-or-false answers about each expression with AND. That is why a type error anywhere makes the whole answer false, and why the thesis can treat "the conservation rule holds" as a *proposition* about a program: the currency-row solver establishes that the proposition is true for every possible input, which is the same discipline — a claim about all rows of a truth table — that a chip designer applies to a circuit before it is manufactured.

That last point is the one to carry into Part II. `conserve per (txn, cur)` is a sentence in the language of §2.2. Its truth is not observed at run time; it is *proved*, over all inputs, before the program is allowed to exist. The distance between "this circuit's output is 1 whenever the inputs are…" and "this transaction balances whenever its legs are…" is the distance between §2 and the rest of this guide, and it is shorter than it looks.

## 3. What a program is, and what "running" means

A program, as a thing on disk, is a file of text. Just characters — letters, brackets, semicolons — with no more inherent power than a shopping list.

Turning that text into the numbered orders of §1 takes one of two routes.

**Compiling** is translating the whole text, in advance, before any of it runs. Rust is compiled all the way down to machine orders. `nilesc`, this project's Niles translator, compiles too, but stops one level short: it produces the intermediate form described in §5 and hands that to an engine that carries it out, rather than emitting processor orders itself. Both are translators producing a finished translation before the clerk sees a word of it; they differ in how far down they translate.

**Interpreting** is having a program read your text and do what it says, line by line, as it goes. It is a translator standing next to the clerk rendering each sentence aloud as it comes. This project has one of these too — `niles-interp` — used for the early stages of the language.

Compiling is faster to run and slower to start; interpreting is the reverse, and it catches fewer mistakes in advance because it has not looked at the whole text before beginning. The difference matters to this project because "catching mistakes in advance" is, more or less, the thesis.

**Running** a compiled program means: the operating system finds a free stretch of pigeonholes, copies the orders into them, points the clerk at the first order, and lets go. The program is then a *process* — a living thing with its own memory, its own position in its own instruction list, and no ability to reach into anyone else's.

## 4. What an abstraction is

This is the most important section in this guide. Everything else is detail.

An **abstraction** is a name for a pile of complexity, agreed on in advance, so that you can use the pile without carrying it around in your head.

That sounds thin. It is the whole of the subject.

**The everyday version.** "Send a letter" is an abstraction. Underneath it: ink, envelope, stamp, postbox, sorting office, van, plane, another sorting office, another van, a person with a bag, a slot in a door. You do not hold that in mind. You hold *send a letter*. And crucially, the pile can change entirely — the plane can be replaced, the sorting office automated — and *send a letter* still means what it meant. You were never depending on the plane. You were depending on the promise.

That is the definition, and it has two halves that are easy to run together.

**The literal abstraction** is the mechanism: the actual ink and vans. In code, it is the actual instructions the processor will actually carry out.

**The meaning of the abstraction** is the promise: give me a letter and an address, and it arrives. In code, it is what the thing guarantees regardless of how it is built.

A program is a tower of these. At the bottom, the processor's add-and-jump. A few levels up, "add these two numbers even though they are bigger than one pigeonhole". Higher, "look up this customer". Higher still, "transfer money between accounts, conserving it". Each level is written using the level below and *sells a promise* to the level above. Someone reading at the top does not know or care what the bottom does.

**Why this is the whole subject.** Almost every serious failure in software is an abstraction whose promise and whose mechanism disagreed. The promise said "transfers conserve money"; the mechanism, in one rare circumstance, did not. Everyone above was entitled to rely on the promise. Nobody above was checking the mechanism. That is what a bug *is*, structurally: a gap between what a name promises and what its pile does.

And that is why this thesis exists. It is an argument that certain promises — money is conserved, currencies do not mix, nobody spends without permission — should be enforced by the *language*, so that a mechanism which fails to keep its promise cannot be written down, rather than being written down and discovered later. Every strange word you are about to meet in Niles is an attempt to move one promise from the honour system into the machinery.

**Where the analogy breaks.** Postal abstractions leak gracefully: if the plane is delayed the letter is late, not wrong. Software abstractions leak catastrophically — a mechanism that fails its promise in one case out of ten million produces a confidently wrong answer, at full speed, with no indication. This is why programmers care so much about things that look like pedantry.

## 5. How a line of text becomes electricity

A translator — a compiler — works in stages. Naming them is useful because this project's own compiler exposes exactly these stages as commands you can run one at a time, and because half the vocabulary in Part V refers to one of them.

Take a single line of Niles:

```niles
let fee: Money<usd> = 125.00 usd;
```

**Stage 1 — lexing** (also "scanning", or "tokenising"). The text is a stream of characters: `l`, `e`, `t`, space, `f`, `e`, `e`… The lexer's only job is to group them into *words* — tokens — and say what kind each is. Out comes: keyword `let`; identifier `fee`; punctuation `:`; type name `Money`; punctuation `<`; identifier `usd`; punctuation `>`; punctuation `=`; **money literal** `125.00 usd`; punctuation `;`.

*Analogy:* reading a sentence in a language you do not speak and correctly identifying where one word stops and the next begins, without yet knowing what any of them mean.

This is where the money-literal rule from Appendix A lives. The lexer is what decides that `125.00 usd` is a single token of a special kind — and what decides that the three letters at the end must be exactly three lowercase letters, which is why `5 widget` cannot be written.

**Stage 2 — parsing.** The flat stream of words is turned into a tree that reflects structure. `let` takes a name, a type and a value; the type `Money<usd>` is a type-with-a-parameter; the value is a literal. Parsing is what decides that `a - b - c` means "(a minus b) minus c" rather than "a minus (b minus c)" — a decision that does not appear anywhere in the text and must come from a written table of precedence.

*Analogy:* diagramming a sentence. Which words are the subject, which the verb, which clause is inside which other clause.

**Stage 3 — resolution and type-checking.** Now meaning starts. *Resolution* asks: this name `usd` — what does it refer to? Which declaration? *Type-checking* asks: do the kinds line up? A value of kind "money in US dollars" is being stored in a box declared to hold "money in US dollars" — fine. Had the line said `125.00 eur`, this is the stage that would refuse it, by name, before anything ran.

*Analogy:* proofreading for sense rather than grammar. "The Wednesday drank the triangle" parses perfectly and means nothing.

This stage is where nearly all of Niles's distinctive value lives. The conservation of money, the refusal to mix currencies, the requirement that permission be presented before an overdraft — all of it is decided here, on the text, before a single instruction runs.

**Stage 4 — lowering to an intermediate representation.** The tree, now checked, is rewritten into a simpler, more uniform form called the **IR**. It is no longer Niles and not yet machine code. It is a small, regular language designed to be analysed and optimised rather than read by people.

*Analogy:* translating a contract from legal English into a numbered list of unambiguous obligations. Nothing is added; ambiguity and ornament are removed.

Why bother? Because everything downstream — checking, optimising, planning — becomes a job on one small regular structure rather than on a large irregular one. And because it is the joint: in this project, both the Niles surface and the SQL surface turn into *the same* IR, which is precisely what makes "the SQL support is a way of speaking, not a second set of meanings" a structural fact rather than a good intention.

**Stage 5 — code generation.** The IR becomes actual processor orders from §1, or — in this project — is handed to a runtime that executes it directly.

**Stage 6 — linking and loading.** Separately compiled pieces are stitched into one file, the operating system loads it, and the clerk starts reading.

The important thing to take away: **a mistake caught at stage 3 costs nothing, and the same mistake caught at stage 6 costs a production incident.** The entire design of Niles is an argument about moving checks left along this pipeline.

## 6. Types: the idea that a value has a kind

The processor sees only numbers (§1). A *type* is a claim, made in the program's text and checked by the compiler, about what a number *means* and therefore what may be done with it.

`42` might be a count of transactions, a temperature, a customer's age, or a position in memory. A type system is a language for saying which, and a mechanism for refusing operations that make no sense across kinds — adding a temperature to a customer's age, or, here, adding dollars to euros.

**The analogy.** Types are the shapes of the holes in a child's shape-sorter. The square peg does not go in the round hole — not because a rule forbids it but because the geometry does not permit it. A good type system arranges for whole categories of mistake to be shaped wrong rather than merely disallowed.

Two ideas that will recur:

**Static versus dynamic.** *Static* means checked before running, by reading the text. *Dynamic* means checked while running, if at all. Rust and Niles are both statically typed, aggressively so. The consequence: a large class of failures becomes a translator's complaint at your desk rather than an incident at 3 a.m.

**Types as promises.** `Money<usd>` is not merely "a number". It is "an exact whole number of cents, in US dollars, which may be added to other US dollars and to nothing else". The type is where that promise is written down, and the type-checker is what enforces it. When this thesis says a category of accident is *unwritable*, this is the mechanism it means.

## 7. Memory: where the values live

Two regions matter, and the distinction explains a lot of Rust.

**The stack** is a pile of plates. When a function starts it puts a plate on top holding its local values; when it finishes the plate is removed. Fast, perfectly ordered, and strictly limited: everything on it must have a size known in advance and a lifetime that matches the function's.

**The heap** is the warehouse. You ask for a space of whatever size, you are given its address, and it stays yours until you give it back. Flexible, slower, and — this is the point — *somebody has to remember to give it back*.

Two failures follow, and between them they account for an enormous share of the world's security vulnerabilities. Forget to give space back and the program slowly consumes all the memory there is (a *leak*). Give it back and keep using the address and you are reading or writing a pigeonhole that now belongs to something else (a *use-after-free*), which is the raw material of a great many exploits.

Languages answer this in three ways. C hands you the responsibility. Java and most modern languages run a *garbage collector* — a mechanism inside the running program that periodically works out what is unreachable and reclaims it, at the cost of pausing occasionally and unpredictably. Rust does something else, and §23 is about what.

This project cares because of the pauses: a system with a latency promise cannot have an unpredictable pause in the middle of an authorization.

## 8. What a bug is, and what a language can do about it

From §4: a bug is a gap between a promise and a mechanism. Languages differ in which gaps they permit.

A mistake can be caught at four moments, and the cost rises by roughly an order of magnitude at each step:

1. **Unwritable.** The mistake cannot be expressed. Adding euros to dollars in Niles is here — there is no operation that accepts one of each.
2. **Refused by the compiler.** It can be written, and the translator refuses it. Most type errors are here.
3. **Caught at run time.** The program notices and stops or complains. Better than nothing; it happens in production.
4. **Silently wrong.** The program continues, confidently, with a wrong answer. This is the one that ends up in a regulator's report.

Every unusual feature in Niles is an attempt to move a specific banking accident from category 4 or 3 up into 2 or 1. That is the design brief. When you meet a word like `conserve` or `authorize` or `resolve` and wonder why a language would have such a thing, the answer is always: because there is a category-4 failure it is trying to make category-1.

---

# Part II — Niles

## 9. What Niles is, and the one rule that shaped it

Niles is a language for describing a bank's data and asking it questions. It is meant to replace SQL — the language nearly every database in the world speaks — in settings where getting the answer wrong is expensive.

It has a stated rule about where its notation comes from, and knowing the rule makes the language far easier to read:

1. **If SQL already has a way of writing it, write it SQL's way.**
2. **Otherwise, if Rust has a way, write it Rust's way.**
3. **Only invent notation for ideas neither language has.**

That is the rule as thesis Appendix B.1 and §6.25 state it — *SQL-first, Rust-fallback, novel-last* — applied construct by construct, with the choice of that order over its inverse adjudicated in Appendix J.1. It carries one stated exception: a SQL spelling is not reused where it would cost the engine determinism or efficiency, which is why unbounded recursion is admitted only behind a guard and `float` is not a legal money type. (The project's one-paragraph description elsewhere says "Rust first"; the thesis's normative text says SQL first, and this guide follows the thesis.)

So a Niles file looks like Rust that has swallowed a database. `select`, `from`, `group by`, `join`, `create`, `grant` come straight from SQL, and keep SQL's meaning wherever that meaning survives. `fn`, `let`, `match`, `struct` come straight from Rust, for the things SQL has no way to say. And then there are sixty-six words that exist nowhere else, because they name ideas nobody had needed to name: `conserve`, `anchor`, `serve`, `evictable`, `declassify`, `resolve`.

**Why bother with a new language at all?** Because of §8. SQL can *express* a bank, but it cannot *refuse* one that is wrong. In SQL, "these two rows must sum to zero" is a comment, a convention, or a trigger that runs after the fact. In Niles it is a declaration the compiler checks before anything runs. The thesis is careful about this claim, and so should you be: it is an argument that the checks belong in the language, and the evidence for it is a counted sixteen obligations over four files. It is a real argument with modest evidence, not a settled fact.

**The shape of a file.** A Niles file has two kinds of thing in it. A `schema` block *declares the world* — what currencies exist, what relations exist, what questions are standing. Outside it are `fn` items — *procedures*, things that happen. Declarations are nouns; functions are verbs.

## 10. Declaring the world

```niles
schema demo_bank {
    currency usd { scale: 2 }
    ...
}
```

`schema` opens the declaration block and gives it a name. Everything inside describes what exists, not what happens.

### Currencies

```niles
currency usd { scale: 2 }
currency jpy { scale: 0 }
```

`currency` declares a currency and — this is the point — its **scale**: how many digits come after the decimal point. Dollars have two (cents). Yen has none. Kuwaiti dinar has three.

*Why a language would care:* because the single commonest money bug in the world is a scale mistake. A system that assumes two decimal places everywhere either loses a third of a Kuwaiti fils or multiplies yen by a hundred. Here the scale is part of the *type*, so `1_000.00 jpy` is not a rounding risk — it is a compile error, refused by name, because yen has no minor unit.

*Analogy:* declaring, at the top of the ledger, that this jar holds coins worth a hundredth of a dollar and that jar holds whole yen. Once declared, no clerk can quietly treat one as the other.

### The three kinds of relation

A *relation* is a table of rows — the fundamental shape of stored data. Niles has three kinds, and the difference between them is the thesis in miniature.

```niles
table accounts { ... }          // mutable, no history
base holds { ... }              // immutable, fully retained
ledger postings { ... }         // immutable, fully retained, and conserved
```

**`table`** is the ordinary kind, as in any database. Rows can be changed with `update` and removed with `delete`. History is not kept. Use it for things whose past does not matter: a customer's current address, a configuration setting.

*Analogy:* a whiteboard. Wipe and rewrite freely.

**`base`** is immutable and fully retained. There is no `update` and no `delete` — those words are not merely discouraged, they are meaningless against it and the compiler says so. Rows are appended, in order, in batches, and each sealed batch advances the world's clock by one tick — an *epoch* (§12). Nothing is ever removed.

*Analogy:* the bound ledger book with no eraser from Appendix A.

**`ledger`** is a `base` plus one extra obligation: a conservation rule. It is the kind for money.

```niles
ledger postings {
    txn: TxnId,
    acct: Id<Account>,
    cur: Currency,
    amt: Money,
    value_date: Date,
    idem: IdemKey window 1_000_000.epochs,
    conserve per (txn, cur);
    retain forever;
    bitemporal;
}
```

Line by line:

- `txn: TxnId` — a column called `txn` holding a transaction identifier. Everywhere in Niles, `name: Type` means "a thing called *name* whose kind is *Type*". That colon is the most frequent punctuation in the language.
- `amt: Money` — an amount. Not a decimal number: a `Money`, which carries its currency and its scale in its type.
- `value_date: Date` — the banking value date. Which day this posting counts for, as distinct from which day it was recorded (§12).
- `idem: IdemKey window 1_000_000.epochs` — see §15 on idempotency. In short: a fingerprint that lets the system recognise the same instruction arriving twice, remembered for a declared length of time.
- `conserve per (txn, cur);` — **the double-entry rule.** For every transaction, and separately within every currency, the amounts must sum to exactly zero. This is the promise from §4 that the whole project exists to move into the machinery.
- `retain forever;` — never evicted. Mandatory on a base or ledger; this is the asymmetry of Appendix A.9 written as a declaration.
- `bitemporal;` — this relation keeps both calendars (§12).

Note what the example's own comments record: `holds` was written as a `ledger` and the compiler refused it, because a hold is one-sided — reserving money is not a movement of it — and so no hold ever sums to zero. That refusal is the language doing its job on its own author.

### Anchor indices

```niles
index by_acct on postings (acct, cur) anchor;
```

An **index** is a lookup aid: a way of finding the rows for one account without reading the whole book. `anchor` marks it as the special kind this engine requires on ledger keys, because reconstruction (Appendix A.10) has to reach a key's rows directly. Without it, filling one gap would mean scanning ten years of history, and the whole economic argument collapses.

*Analogy:* the tabbed dividers in a ring binder. Without them, "find everything about account 7" means turning every page.

### Views

```niles
view ledger_balance = postings
    .group_by(|p| (p.acct, p.cur))
    .sum(|p| p.amt)
    serve { consistency: read_your_writes, materialize: auto, retain: evictable };
```

A **view** is a standing question with a name. This one says: take the postings, gather them into groups by (account, currency), and within each group add up the amounts. That is what a balance *is* — not a stored number but a derived one.

The view is the central object of the whole thesis. In the theory it is called a **REV** — a *reconstructible epoch-anchored view*. Reconstructible: it can always be rebuilt from the ledger. Epoch-anchored: every answer it gives carries the moment it is true at.

The `serve { ... }` block is the **contract** (§14) — and it is part of the view's *type*, not a configuration file somewhere else. A view's freshness promise travels with the view.

## 11. Money

```niles
let rent = 850.00 usd;
```

A **money literal**. `850.00` is the amount, `usd` is the currency, and the two are one token — you cannot have the number without the currency. The fractional digits must match the declared scale exactly: `2.50 usd` is well-formed, `2.5 usd` is a compile error, `1_000.00 jpy` is a compile error.

*The underscore* in `1_000_000` is a thousands separator that the compiler ignores. It exists purely so a human can see at a glance that a number is a million and not a hundred thousand. Both languages in this guide allow it between the digits of a number.

The type is written `Money<usd>` — "money, in dollars". The currency is *inside the type*, which is what makes the central promise work:

- `Money<usd> + Money<usd>` → fine.
- `Money<usd> + Money<eur>` → **there is no such operation.** Not forbidden by a rule; simply not defined. This is category 1 of §8.
- `Money<usd> * 3` → fine, an integer scaling.
- `Money<usd> * Money<usd>` → meaningless and absent. Dollars times dollars is square dollars.
- Any floating-point number anywhere near a `Money` → refused by type.

That last one deserves a moment. Floating-point numbers are the normal way computers hold fractional values, and they are *approximate*: the value written `0.1` is not exactly a tenth, and adding a tenth ten times does not give exactly one. Over a billion transactions that approximation becomes real money. Niles holds money as an exact whole number of minor units — cents, yen, fils — and bans floats from money positions *by type*, not by convention.

*Analogy:* counting with coins instead of measuring with a ruler. Coins are exact and there is no such thing as most of one.

**`fx` — crossing currencies.** Since no operation joins two currencies, an exchange cannot be a conversion. It is written as two separate balanced movements sealed together:

```niles
fx {
    leg usd_side: post(debit(a, 1000.00 usd)?, credit(b, 1000.00 usd)),
    leg eur_side: post(debit(b, 920.00 eur)?, credit(a, 920.00 eur)),
    rate: r,
}
```

`fx` is the form; `leg` is one side of it; `rate` records the conversion used. Dollars leave one account and arrive at another, and *separately* euros leave and arrive. Both currencies conserve independently, which is the only arrangement in which "no jar's total is ever wrong" survives a crash halfway through.

The example's comment records that the thesis's own printed version of this had the effects wrong — it claimed to debit dollars and credit euros, which conserves neither — and that the compiler caught it.

## 12. Time

Time in Niles is three separate ideas that most systems confuse.

### Epochs

```niles
#41209
```

An **epoch** is a tick of the system's own clock: a batch of writes sealed together. It is a counter, not a wall-clock time. Epoch 41,209 is the 41,209th sealed batch, and it means exactly the same thing on every machine for ever, which is the point — wall-clock times differ between machines, drift, and go backwards when a clock is corrected.

The `#` prefix marks an epoch literal. `3.epochs` is a *duration* in epochs.

*Analogy:* page numbers in the vault book. "As of page 41,209" is unambiguous in a way that "as of about half past two" is not.

### The two calendars

```niles
recorded_at    // when we wrote it down     (system time)
valid_at       // when it was true          (valid time)
```

A relation declared `bitemporal` carries both. `recorded_at` is never rewritten — it is what actually happened to the bank's knowledge. `valid_at` is what happened in the world, and it may be in the past relative to when we learned it.

`as_of` pins a read to a point on the system axis; `valid_at` pins it on the world axis. Together they answer the auditor's question: *what did you believe on Wednesday about what was true on Monday?*

`value_date` is the banking-specific version: the date a posting counts for, which drives its placement on the valid-time axis. In a real bank this is not a nicety — European payments law constrains the relationship between the two dates.

**Temporal literals.** `@2026-08-25T12:00:00Z` is an instant; `@2026-08-25` a date; `v@2026-08-25` a **value date** specifically; `@2026-01-01 ..= @2026-12-31` a closed interval. The `@` sign is the marker that says "what follows is a moment".

### Signals

```niles
Signal<T>
```

A **signal** is a value that is a function of time rather than a single number: *balance as a function of epoch*, not *balance*. Ask it at any epoch and it gives the value there. This is how "what was this on that date three years ago" stops being a special report and becomes an ordinary read.

*Analogy:* the difference between a photograph and a film. Most systems store the photograph and try to reconstruct the film from a change log. Here the film is the primary object.

## 13. Asking questions: the pipeline

Niles writes queries as a **pipeline** — a chain of steps, each feeding the next, read top to bottom:

```niles
postings
    .where(|p| p.value_date >= v@2026-08-01)
    .group_by(|p| p.acct)
    .sum(|p| p.amt)
```

Read it aloud: *take the postings; keep the ones whose value date is on or after the first of August; gather them into groups by account; within each group, add up the amounts.*

Two pieces of notation are doing work here.

**The dot.** `x.f(...)` means "apply `f` to `x`". Chaining dots chains steps, and because each step's output is the next one's input, the order you read is the order it happens. SQL famously does not have this property: a SQL query begins with `select`, which is nearly the last thing that happens. Niles offers SQL's vocabulary in SQL's order inside `sql { }`, and its own vocabulary in execution order outside it.

**The closure.** `|p| p.amt` is a small anonymous function: "given a row, which I will call `p`, produce `p.amt`". The vertical bars hold the parameter list. `|p| (p.acct, p.cur)` produces a *pair*.

*Analogy:* an instruction to a clerk written on a slip, with a blank where the row goes. "For each row, look at ___ 's amount column." The `p` is the name you give the blank so you can refer to it.

There is also `|>`, an explicit pipeline stage marker, for when the chain is clearer written out than with dots.

The `sql { ... }` block is the escape hatch:

```niles
view sql_balances = sql {
    select acct, cur, sum(amt) as balance
    from postings
    group by acct, cur
} serve { ... };
```

Same result, same internal form, SQL's syntax. The point of the block is that it is *demarcated*: you can see exactly where the old language starts and stops, and the meanings are the same on both sides of the boundary.

## 14. The serve contract

```niles
serve {
    consistency: ledger_consistent,
    materialize: demand,
    retain: evictable,
    budget: 50000,
    lineage: key,
}
```

A view's contract has eight terms. It is the most distinctive thing in the language and has no counterpart in SQL at all. Most views declare only the three or four that matter to them; the rest take their defaults. The eight are `consistency`, `checkpoint`, `materialize`, `budget`, `freshness`, `retain`, `lineage` and `backfill`, and confidentiality is inherited from the column annotations rather than declared here. The five that carry the most meaning are below; `checkpoint` sets the bookmark interval of Appendix A.13, `freshness` the staleness bound of a `bounded` rung, and `backfill` how far back a newly declared view is populated from history.

### `consistency` — which rung of the ladder

How fresh must the answer be? Six values, loosest to strictest:

| Value | Promise |
|---|---|
| `bounded(epochs: K, millis: T)` | At most K ticks behind and no older than T milliseconds, whichever binds first. |
| `monotonic` | Within a session, answers never go backwards in time. |
| `read_your_writes` | You always see at least your own committed writes. |
| `snapshot` | Everything in one read comes from one single moment, across keys and views. |
| `serializable` | Reads and writes together fit one serial order. |
| `ledger_consistent` | Anchored at the newest visible tick; reflects every write that finished before the read began. Strict serializability. |

The vital point, easy to miss: **correctness is not on this list.** Every rung gives an answer exactly equal to a system that never evicted anything. The rungs differ only in *which moment* you are told about. A balance served at `bounded` is not an approximate balance — it is an exact balance, of a slightly older moment, and it is stamped as such.

*Analogy:* the difference between a wrong answer and a correct answer to a slightly earlier question. The first is a defect; the second is a choice you can price.

The example is worth studying: `ledger_balance` is served at `read_your_writes`, and `available_balance` — the one a card authorization consults — at `ledger_consistent`. And the comment records that `available_balance` originally derived *from* `ledger_balance` and the compiler refused it, because staleness entered one step upstream and no downstream promise can remove it. That is a real class of banking failure — two views of one ledger disagreeing at the moment of a decision — caught by a type rule.

### `materialize` — how much is kept

| Value | Meaning |
|---|---|
| `full` | Everything is kept, always. |
| `demand` | Keep only what has been read; evict the rest; rebuild on demand. |
| `absent` | Keep nothing; every read reconstructs. |
| `spilled` | Kept, but on slower storage. Illegal at the strictest rung. |
| `tiered` | Hot entries in fast memory, cold ones spilled. |
| `auto` | Let the optimizer decide, within the rest of the contract. |

`demand` is the mode the thesis is about. `auto` delegates to a component that, as Appendix A.15 says plainly, does not exist yet; `spilled` and `tiered` likewise describe places the design has and the implementation does not.

### `retain` — may this be dropped?

`evictable` (may be dropped and rebuilt), `pinned` (resident always, at a declared memory cost), `forever` (never dropped — mandatory on a base or ledger).

### `budget` — the ceiling

A number of entries or bytes. The view's resident state may not exceed it. This is the "notebooks are small" constraint made explicit and enforceable.

### `lineage` — how much provenance

`off`, `key`, or `full`. Even `off` still stamps every answer with its anchor. Read Appendix A.19 before relying on the other two: the component that would deliver them was withdrawn.

## 15. Writing: transactions, postings, holds

### `txn`

```niles
txn idem("rent-2026-08") {
    let d = debit(tenant, rent)?;
    let c = credit(landlord, rent);
    post(d, c)
}
```

`txn` opens a **ledger transaction**: the unit over which the conservation rule is checked. Everything inside happens together or not at all, and at the closing brace the amounts must sum to zero per currency or the whole thing is refused.

`idem("rent-2026-08")` is the **idempotency key**: a name for this instruction. If the same instruction arrives twice — a retried network request, a duplicated message, a customer double-tapping — the system recognises the name and does not do it twice. The window over which the name is remembered is declared on the relation (`window 1_000_000.epochs`), so "how long do we deduplicate for" is a written, checkable property rather than a thing someone hopes about.

*Analogy:* writing a reference number on a cheque. Present the same cheque twice and the second one is recognised, not honoured.

### Postings, debits and credits

```niles
let d = debit(tenant, rent)?;
let c = credit(landlord, rent);
post(d, c)
```

A **posting** is one signed movement. A `Debit<usd>` and a `Credit<usd>` are its two halves, and they are **linear**: each must be used exactly once. Not zero times — you cannot create a debit and then forget about it, because a half-written movement that quietly vanishes is exactly the kind of silent error §8 exists to make unwritable. Not twice — you cannot post the same debit into two transactions.

Linearity is enforced by the type system, and it is the most unusual idea in the language. In most languages a value can be copied and dropped freely. Here, dropping a `Debit` without posting it is a compile error.

*Analogy:* a physical banknote rather than a number written on a form. You cannot photocopy it and you cannot leave it on the table — it has to go somewhere, exactly once.

### Holds

```niles
let h = hold(acct, amount, expires: 7.days)?;
resolve h post 18.50 usd
```

A **hold** is a reservation: money set aside before it moves, which is why the balance you have and the balance you can spend differ. In this design it is a ledger fact of its own, not a note in a margin.

A hold is also linear, and it has exactly three endings, all spelled with `resolve`:

- `resolve h post 18.50 usd` — capture up to the held amount.
- `resolve h void` — cancel; release the reservation with no posting.
- `resolve h expire` — the window lapsed; release it.

The type system rejects a program that resolves a hold twice, and rejects one that lets a hold go out of scope unresolved. A forgotten authorization hold — money reserved against a customer's account and never released — is a real and common banking complaint, and this is it made unwritable.

## 16. Permission

```niles
capability overdraw;

fn debit(acct: Id<Account>, amt: Money<usd>, auth: Option<Auth<overdraw>>) -> ...
```

A **capability** is an unforgeable token authorising a specific effect. `Auth<overdraw>` is a value that *is* the permission to overdraw. You cannot fabricate one; you can only be handed one.

`authorize` is the only construct in the language that may take an account past its floor, and it requires such a token.

*Why this rather than roles?* Most systems ask "does this user have the manager role?" at the moment of the action. That is a question about ambient state, and the failure mode is forgetting to ask. Here the permission is a *parameter*: a function that can overdraw has, in its signature, a slot for the permission to do so, and a caller who has no token cannot call it. The check cannot be forgotten because there is nowhere to forget it from.

`grant` confers a capability; `revoke` withdraws one, and — note — a revocation is recorded as a ledger event, never a silent edit.

The honest limit, from Appendix A.20: this guarantees nobody *forgot to ask*. It does not guarantee you never go overdrawn. The asking still has to happen, and the clause of the soundness theorem covering this is narrower than its first draft.

## 17. Secrecy

```niles
owner: Text @confidential(e2ee, subject = id),
```

The `@confidential(...)` annotation marks a column as encrypted, with three levels:

- `e2ee` — end-to-end encrypted. The engine may not compute on it at all: no filtering, no joining, no aggregating. It stores and orders and fingerprints an opaque box.
- `committed` — a middle level, less blind than `e2ee`: *an additively homomorphic commitment, usable in sum-checks only* — a sealed value that can still be added up without being opened, so the engine can verify that a column of amounts totals what a ledger says it should without learning any one of them. Until cycle 13 the compiler's keyword registry described this level as something else entirely (*readable only inside the enclave that holds the key*, which is a different mechanism with a different threat model, and is in any case what `e2ee` says); the registry now carries Appendix B.13's definition and the two agree. **The level has no mechanism behind it, and the compiler does not even distinguish it from `e2ee`.** The check that enforces confidentiality (diagnostic `NL0260`) refuses *any* confidential column used in a filter, a key or a total, without looking at which level was written — so a `sum` over a `@confidential(committed)` column, which is the single operation this level exists to allow, is refused with a message saying the engine cannot compute on an encrypted column. It is written down here because a reader meeting the keyword should know it names a design and not a mechanism.
- open — the default; ordinary readable data.

`declassify(x, auth)` is the single construct that lowers a level, and it is always audited and always requires a capability.

The labels are **static**: a column's confidentiality is fixed in its declaration and checked at compile time, so a query that tries to filter on an `e2ee` column is refused by the translator rather than failing mysteriously later.

Read Appendix A.27 for what is built: the declarations and their checking are real; the `committed` machinery is design.

## 18. Audit

```niles
explain available_balance.get((acct, usd))?;   // how was this answer derived?
reproduce (answer, #41209);                     // recompute it and compare
impact of postings.row(id);                     // which views does this row affect?
```

Three forms. `explain` looks backwards from an answer to its sources. `reproduce` re-derives an answer from the base and compares — an audit act, performed on demand. `impact` is the inverse of `explain`: given a ledger row, which derived answers does it touch. That last one is what you run when you discover a bad entry and need to know what it contaminated.

`anchor` is the epoch stamp that every answer carries, and the mandatory index kind on ledger keys. Both meanings are the same idea: an answer, and the moment it is true at, travel together.

## 19. The imperative part

Outside the `schema` block, Niles is essentially Rust. `fn` declares a function; `let` binds a value; `if`, `match`, `for`, `while`, `loop` do what Part III describes. The novel additions are `txn` (§15) and the effect row:

```niles
fn pay_rent(tenant: Id<Account>, landlord: Id<Account>) -> Result<TxnId, TxnError>
    ! { append, debit<usd>, credit<usd> }
```

Everything after the `!` is the **effect row**: a declaration, in the function's type, of what it does to the world. This one appends to the ledger and moves dollars in both directions. A caller can see from the signature alone that this function writes.

*Analogy:* a form that states on its face which departments it touches, so that nobody has to read the procedure to find out whether filing it moves money.

**User-defined functions.**

```niles
#[udf(fuel = 1_000_000, deterministic)]
fn risk_score(history: &[Anchored<Money<usd>>]) -> u32 { ... }
```

A `udf` is custom logic that a bank supplies. It runs in a sandbox with a **fuel** limit — a hard ceiling on how much work it may do before being stopped — and no access to the clock, the network or the disk. `deterministic` is checked rather than trusted: operations that could produce different results on different machines are rejected.

*Why so strict?* Because of reconstruction. If a view's definition could consult the wall clock, then rebuilding an evicted entry could legitimately produce a different answer from the one that was evicted — and no audit could distinguish that from a bug. The example file records exactly this: a predicate using `month_start()` was rejected, because a view boundary must be a value, not a moment.

**Guarded recursion.**

```niles
guard measure(remaining)
```

Recursion — a definition that refers to itself — can fail to stop. `guard measure(...)` names a quantity that must strictly decrease on each step, which is a proof that it terminates. Unguarded recursion is rejected. `break`, `loop` and `while` are not permitted inside a view body at all, for the same reason: a standing question must be answerable in bounded time.

---

# Part III — Rust

## 20. Why the engines are written in Rust

Niles is the language the bank writes. Rust is the language the *machine underneath* is written in. Nilestream — the database engine — and GBS — the core-banking platform — are both Rust, and together they are the great majority of the code in this project. If you are going to read one of these repositories, you are mostly going to be reading Rust.

Rust exists to solve the problem in §7. C lets you manage memory by hand and lets you get it wrong, which is where a large fraction of the world's security vulnerabilities come from. Java and its relatives run a garbage collector, which is safe but pauses unpredictably — and an unpredictable pause in the middle of a card authorization is exactly what a latency promise cannot absorb.

Rust's answer is to prove the memory is handled correctly *at compile time*, and then emit code with no collector at all. You get C's speed and predictability with an enforced guarantee that the whole use-after-free family of bugs is absent. The price is that you must explain your intentions to the compiler in a way no other mainstream language demands, and §23 is about that price.

Two facts about this project worth carrying into the rest of Part III. There is **no `unsafe` code** in either engine — the escape hatch Rust provides for stepping outside its guarantees appears only in a memory-measurement tool, one copy per repository, each excluded from its workspace and linked into nothing that ships, and two tests exist whose only job is to fail if that ever stops being true (§32 has the detail). And **no `async`/`await`** — the concurrency machinery — which is a deliberate choice recorded in the compiler's own keyword table: query and transaction context is synchronous by design.

## 21. Values and bindings

```rust
let budget = 100;
let mut count = 0;
const MAX_FLIGHTS: usize = 256;
static PLAN: OnceLock<BasePlan> = OnceLock::new();
```

**`let`** introduces a name for a value. Read it as "let *budget* be 100". The name is a label on a value, not a box you can refill.

**`mut`** makes it a box you can refill. Without `mut`, a `let` binding cannot be changed after it is created — and that is the *default*, which is the opposite of nearly every other language. You must ask for the ability to change something.

*Why:* because most values never need to change, and a value that cannot change is a value no other part of the program can alter behind your back. Making immutability the default turns "can this change underneath me?" from a research question into a glance at the declaration.

*Analogy:* writing in ink by default and having to explicitly ask for a pencil.

**`const`** is a value fixed at compile time and substituted wherever it is used. `MAX_FLIGHTS` is not a variable holding 256; it *is* 256, with a name, so that the number appears once instead of in nine places.

**`static`** is a single value that lives for the entire run of the program, at one fixed address. A `static mut` — one that can be overwritten in place — cannot be touched without `unsafe`, so this project has none. What it uses instead is what the example shows: a `static` holding a `OnceLock`, which can be filled in exactly once and then only read, or a `Mutex`, which hands out one writer at a time. Both are mutation with the guarantees kept, and both are ordinary safe Rust.

**Type annotations.** `let x: u64 = 5;` says explicitly that `x` is an unsigned 64-bit integer. Usually you can omit it — Rust *infers* the type from context — and the codebase omits it where it is obvious and states it where it is not. The colon means "of type", exactly as in Niles.

**Shadowing.** You may `let` the same name twice; the second hides the first. This is used deliberately for a value that changes form: `let text = read()?; let text = text.trim();`.

## 22. The built-in types you will meet

The first six rows are Rust's true primitives; `String` and `Vec` are library types so common that they belong in the same table.

| Written | Is |
|---|---|
| `i8` `i16` `i32` `i64` `i128` | Signed whole numbers, of that many bits |
| `u8` `u16` `u32` `u64` `u128` | Unsigned (non-negative) whole numbers |
| `usize` / `isize` | Whole numbers the size of a memory address on this machine; the type of lengths and indices |
| `f32` `f64` | Approximate fractional numbers. **Never used for money.** |
| `bool` | `true` or `false` |
| `char` | One Unicode character |
| `String` / `&str` | Owned text (a library type) / a borrowed view into text (`str` is primitive) |
| `()` | The *unit* type: no information. What a function returns when it returns nothing |

The bit-width matters: an `i64` can hold about ±9.2 quintillion, a `u8` only 0–255. Adding 1 to a `u8` holding 255 cannot give 256, and what happens instead depends on how the program was built: in a debugging build Rust stops the program on the spot, and in an ordinary optimised build it wraps silently round to 0. That second case is a category-4 failure from §8, and it is the reason a codebase that cares writes `checked_add` — which returns "did it fit?" as a value you must handle — rather than a bare `+`. This one does: `crates/niles-ir/src/arith.rs` turns every overflow into an `ArithError` the caller has to answer, and the interpreter and prototype engine do the same on their balance slots.

Two from this project: `Value = i128` — the engine's amounts, a 128-bit whole number of minor units, because exactness matters more than range; `Key = Vec<i64>` — a view's group key is a *list* of numbers, because you may group by account and currency and desk at once.

## 23. Ownership — the hard chapter

This is the idea that makes Rust Rust. It takes a page and it is worth the page.

**The rule: every value has exactly one owner, and when the owner goes out of scope the value is freed.**

```rust
let a = String::from("hello");   // `a` owns this text
let b = a;                        // ownership MOVES to `b`
// `a` can no longer be used — the compiler refuses
```

In most languages, `let b = a` gives you two names for one thing, and the question of who cleans it up is answered either by you (C — and you can get it wrong) or by a garbage collector (Java — at the cost of a pause). Rust answers it structurally: there is one owner, always, and cleanup happens exactly when that owner's scope ends. No collector, no leak, no double-free, and **it is all decided while compiling**. At run time there is nothing to check.

*Analogy:* a physical key to a room rather than a copy of one. Hand the key to someone else and you do not have it any more. There is never a question of who locks up, because there is never more than one key.

This is why linear types in Niles (§15) felt natural to add: Rust already works this way for everything, and `Debit` merely tightens it from "at most once" to "exactly once".

**`move`** makes the transfer explicit, chiefly for closures: `move |x| ...` means the closure takes ownership of what it captures rather than borrowing it.

**`Clone` and `Copy`.** Some types are cheap enough to duplicate freely (`Copy` — integers, booleans); for others you ask explicitly with `.clone()`, and the explicitness is deliberate, because a copy of a large structure is a cost you should be able to see in the text.

**`drop`** is what happens automatically when an owner's scope ends. A type can define what "being dropped" means, and this project uses that: `FoldTicket` has a `fn drop(&mut self)` so that abandoning a reconstruction partway through cleans up after itself, no matter how the function exits.

## 24. Borrowing: `&`, `&mut`, and lifetimes

Moving ownership every time you wanted to *look* at something would be unbearable. So you can lend instead.

```rust
fn answers(&self) -> &BasePlan
```

**`&T`** is a **shared reference**: a borrow that can read but not change. You may have any number of these at once.

**`&mut T`** is an **exclusive reference**: a borrow that can change. You may have exactly one, and while it exists there may be no shared references at all.

**The rule: many readers, or one writer, never both.**

That single rule eliminates *data races* — two things touching one value at the same time with at least one of them writing, which is the most vicious category of concurrency bug, because it is timing-dependent and vanishes when you look at it. In Rust it is a compile error.

*Analogy:* a library book. Many people may read the reference copy at once. Only one person may take it away to write in it, and while they have it nobody may read it. The librarian enforces this at the desk, before you leave, not after you have caused a problem.

This is the mechanical basis for the thesis's memory-safety claim, and it is why Appendix A.17 can say the *design* removes whole categories of collision even though the throughput has not been demonstrated.

**Dereferencing: `*`.** If `r` is a reference, `*r` is the value it refers to. In a filter you will see `**o == Op::Crash` — two stars because there are two layers of borrow to see through.

**Lifetimes: `'a`, `'static`.** The apostrophe marks a *lifetime* — a name for "how long this borrow is valid". Usually the compiler works it out silently. Where it cannot, you write it: `fn all_names() -> &'static [&'static str]` returns a borrow that is valid for the entire run of the program, which is true because it points at text baked into the executable.

*Analogy:* a note on a loan saying when the book is due back. Most of the time everyone knows. Occasionally you must write it down so that nobody schedules a reading for after the return date.

## 25. Building your own types

### `struct` — a thing with named parts

```rust
pub struct Amount {
    pub minor: Minor,
    pub currency: Currency,
    pub scale: u32,
}
```

A **struct** groups several values into one named thing. This one is the engine's money: an exact whole number of minor units, the currency, and the scale. *All three together* are what an amount is, and bundling them means you cannot pass the number around without the currency — the same discipline Niles enforces in its literals.

*Analogy:* a form with labelled boxes. A blank form is the type; a filled-in one is a value.

**Tuple structs** name the type but not the fields: `pub struct AccountId(String);` — an account identifier which *is* a piece of text, but is not interchangeable with any other piece of text. This is enormously useful: `fn transfer(from: AccountId, to: AccountId)` cannot be called with a customer's name by mistake, whereas `fn transfer(from: String, to: String)` can.

**Unit structs** have no data at all: `pub struct AllDays;`. They exist to *be* something — a marker, a choice of behaviour — rather than to hold anything.

### `enum` — a thing that is one of several possibilities

```rust
pub enum Slot<V> {
    Bottom,
    Hole(Epoch),
    Pending(Epoch),
    Present(V, Epoch),
}
```

An **enum** says: this value is exactly one of these cases, and each case may carry its own data. This one is the heart of Appendix A.9's distinction, and it has four cases, ordered from knowing nothing to knowing everything:

- `Bottom` — *nobody ever asked.* No value has ever existed here, and not even a version is known.
- `Hole(Epoch)` — *we worked it out and later rubbed it out.* The value is gone; the epoch it was last certified through is kept, as an audit fact.
- `Pending(Epoch)` — *someone is fetching it right now.* A reconstruction is in flight.
- `Present(V, Epoch)` — the value, and the epoch it is true at.

Most languages would express this with a number and a comment, or with a null and a convention, and the difference between *never asked* and *asked and forgotten* would live in somebody's head. Here it is a type.

And — see §27 — the compiler will refuse a piece of code that forgets one of the cases.

*Analogy:* a form with tick-boxes where exactly one must be ticked, and each box has its own follow-up question.

`ReadOutcome` is the other one you will meet constantly, and it has three cases: `Hit(Anchored)` — the answer was resident, here it is with its epoch; `Fold(FoldTicket)` — it was not, and here is your claim on the reconstruction you must now perform; `Join(WaitTicket)` — somebody is already fetching this key *at this exact epoch*, so wait for theirs. The third case is Appendix A.12: joining is on the pair, never on the key alone.

### `type` — an alias

```rust
type Key = Vec<i64>;
```

A second name for an existing type. Not a new type — `Key` and `Vec<i64>` are interchangeable — just a name that says what it is *for*.

## 26. Behaviour: functions, traits, generics

### `fn`

```rust
fn append(&mut self, set: &Sealed) -> Result<Epoch, LedgerError> {
    set.verify().map_err(LedgerError::Kernel)?;
    ...
    Ok(e)
}
```

`fn` declares a function. The parameters are `name: Type`, the `->` gives the return type. Read the arrow as "produces".

`&mut self` as the first parameter means this function is a *method* on some type and may modify it. `&self` means it may only read. No `self` at all means it is an associated function called on the type itself, like `Amount::new(...)`.

### `impl` — attaching behaviour to a type

```rust
impl Amount {
    pub fn new(minor: Minor, currency: Currency, scale: u32) -> Self { ... }
}
```

`impl` opens a block of functions belonging to a type. `Self` (capital S) means "the type we are implementing", so `-> Self` is "returns an `Amount`" without repeating the name.

### `trait` — an interface, a contract

```rust
pub trait Base {
    fn answers(&self) -> &BasePlan;
    fn frontier(&self) -> Epoch;
    fn reconstruct(&self, k: &Key, anchor: Epoch) -> (Value, u64);
    fn deltas_at(&self, e: Epoch) -> Vec<(Key, Value)>;
    fn delta_rows_at(&self, e: Epoch) -> u64;
}
```

A **trait** is a list of capabilities a type may claim to have. Anything that can say which questions it answers, report how far it has advanced, rebuild a value at a given moment, hand over one epoch's changes, and say how many rows that epoch touched *is a `Base`*, whatever else it is.

```rust
impl LedgerSink for MemoryLedger { ... }
```

`impl X for Y` — "`Y` has capability `X`" — and now `Y` may be used anywhere an `X` is wanted.

*Analogy:* a professional qualification, not a job. "Certified to sign accounts" is a trait; the accountant, the auditor and the software are different types that all hold it, and anywhere the certificate is required, any of them will do.

This is the joint the engine turns on. `Rev` — the view — does not know what a ledger is. It knows something that implements `Base`, which is why the same view machinery works over the real hash-chained ledger, over an in-memory test fixture, and over GBS's journal.

### Generics: `<T>`

```rust
pub enum Slot<V> { ... }
fn largest<T: Ord>(items: &[T]) -> &T
```

The angle brackets hold a **type parameter** — a blank to be filled in later. `Slot<V>` is a slot holding *some* type, and `Slot<i128>` is that blank filled with `i128`. Write the machinery once; use it at every type.

`T: Ord` is a **bound**: `T` may be any type, provided it can be ordered. A `where` clause is the same thing written after the signature when it grows long.

*Analogy:* a form printed with "quantity of ______". The form is generic; a particular filled copy is concrete. The bound is the small print: "must be something countable".

**The turbofish, `::<>`.** When the compiler cannot infer which type you meant, you say so: `raw_digits.parse::<u64>()` — "parse this text, as a `u64` specifically". The name is a joke about its shape; it is the standard term.

### `dyn` and `impl Trait`

`&dyn Fn(&str) -> Option<u32>` — `dyn` means the exact type is decided while running, and the program follows a pointer to find the right code. Flexible, slightly slower.

`-> impl Iterator<Item = &Row>` — "returns something that iterates; I am not telling you what". Decided at compile time, so no cost at all, but the caller cannot name the type.

`dyn` is *forbidden* in Niles view bodies (§35), because a view must be planned statically: an engine that cannot see, before running, which code a step will use cannot plan the walk backwards of Appendix A.10.

## 27. Choosing between cases

### `match`

```rust
match outcome {
    ReadOutcome::Hit(a)   => serve(a),
    ReadOutcome::Fold(t)  => fold_and_install(t),
    ReadOutcome::Join(w)  => wait_for(w),
}
```

`match` compares a value against patterns and takes the first that fits. `=>` separates the pattern from what to do.

**It must be exhaustive.** Leave out the `Join` arm and this does not compile. If `ReadOutcome` grows a fourth case tomorrow, every `match` in the codebase that does not handle it stops compiling. This is one of the most valuable things in the language: adding a case to a type produces a complete list of the places that need attention, rather than a set of silent wrong answers.

*Analogy:* a decision table that the auditor refuses to accept until every row has an entry.

`_` is the wildcard: "anything else". Useful, and worth being suspicious of — a `_` arm is where a future case will go to die quietly.

**Guards** add a condition: `Row::Post(p) if p.acct == a && p.cur == c => Some(p.amt),` — match a posting, *and only if* it is for this account and currency.

**`@` bindings** name a value while also matching its shape: `Tok::Kw(k @ (Kw::Const | Kw::Static))` — "a keyword token which is either `const` or `static`, and call it `k`".

### `if`, `if let`, `let ... else`

`if` is the ordinary conditional, and in Rust it is an *expression* — it produces a value, so `let x = if c { 1 } else { 2 };` is normal.

`if let Row::Post(ref mut post) = ... { }` — "if this value has that shape, unpack it and run this block". A one-case `match`.

`let Ok(entries) = std::fs::read_dir(&dir) else { return; };` — "unpack this, and if it does not have that shape, take this exit". The `else` block must leave — `return`, `break`, `continue` or panic; the compiler checks that it cannot fall through. It keeps the successful path flat instead of indenting the whole function inside a conditional.

## 28. When things fail: `Option`, `Result`, `?`

Rust has no `null`. The billion-dollar mistake — a value that might be absent, indistinguishable from one that is present, until it is used — is simply not available.

**`Option<T>`** is an enum with two cases: `Some(value)` or `None`. If a thing might be absent, its type says so, and the compiler will not let you use it without deciding what to do about the absence.

**`Result<T, E>`** likewise: `Ok(value)` or `Err(problem)`. A function that can fail says so in its return type. You cannot ignore a `Result` by accident.

**`?`** is the shorthand that makes this bearable:

```rust
set.verify().map_err(LedgerError::Kernel)?;
```

"Do this; if it worked, carry on with the value; if it failed, stop here and return the failure to my caller." Without `?`, error handling swamps the logic. With it, the happy path reads straight down the page and every `?` is a visible marker saying *this step can fail and the failure goes up*.

*Analogy:* a procedure where each step ends "…and if this is refused, return the file to your supervisor with the reason attached". The `?` is that sentence, one character long.

**`panic!`** is the other kind of failure: the unrecoverable one. It stops the thread — unwinding back out through every function on the way, so that each owner's `drop` still runs and nothing is left half-open. `unwrap()` and `expect("...")` say "I am certain this succeeded; panic if I am wrong" — fine in tests, a deliberate decision in shipping code.

## 29. Collections, iterators, closures

**`Vec<T>`** — a growable list. **`BTreeMap<K, V>`** — a lookup table kept in sorted order (the engine's `ZSet` is one). **`HashMap<K, V>`** — a lookup table with no order but faster access. **`&[T]`** — a *slice*: a borrowed window onto part of a list.

**Closures** are anonymous functions written between vertical bars, exactly as in Niles: `|c| c.text.clone()`.

**Iterator chains** are how nearly all data processing is written:

```rust
f.ops.iter().filter(|o| **o == Op::Crash).count()
```

*Take the operations; look at each in turn; keep only the crashes; count them.* Each step is lazy — nothing happens until the final step asks for results — so the whole chain makes one pass.

**Ranges.** `0..10` is ten numbers starting at zero, *excluding* ten. `0..=10` is eleven, *including* ten. The `=` is the difference between "up to" and "up to and including", and confusing them is the classic off-by-one error.

## 30. Organisation: modules and paths

```rust
mod rev;
use crate::absence::{Epoch, Slot};
pub fn slots_len(&self) -> usize
```

**`mod`** declares a module — a named drawer. **`use`** brings a name into scope so you can write `Epoch` instead of `crate::absence::Epoch` every time. **`::`** separates the parts of a path, like `/` in a folder path.

Four path roots: **`crate`** — the top of this compilation unit; **`super`** — the enclosing module; **`self`** — this one; and an external crate's name.

**`pub`** makes an item visible outside its module. Without it, everything is private by default — another instance of Rust's habit of making the restrictive choice the one you get for free.

A **crate** is a unit of compilation and distribution: one library or one program. This project has fourteen in the Nilestream workspace and four in GBS. The `Cargo.toml` file at the root lists them, and — a detail that has caused this project three separate defects — a crate listed as `exclude`d is *not* built by a command that says "build everything", which is exactly how a broken crate can hide in plain sight.

## 31. Attributes and macros

**`#[...]`** is an **attribute**: a note attached to the item below it.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome { Ok, Failed, NotRun }
```

`derive` asks the compiler to write routine code for you — here: how to print this for debugging, how to duplicate it, how to compare two for equality. Writing those by hand for every type would be pages of drudgery.

`#[test]` marks a function as a test. `#[udf(fuel = ..., deterministic)]` is Niles's version, marking a sandboxed function.

**`#![...]`** with the exclamation mark applies to the *enclosing* item rather than the following one — usually the whole file: `#![allow(dead_code)]`.

**Macros** are called with a `!`: `assert_eq!(a, b)`, `format!("{x}")`, `vec![1, 2, 3]`. A macro is code that writes code — expanded early in compilation, after the text has been split into tokens and before anything is type-checked — which is why it can do things a function cannot — `assert_eq!` prints the text of both expressions when they differ, which requires seeing the source.

`macro_rules!` defines one. This project has four, all small.

**A note on `!`, which has four unrelated jobs:** negation (`!x` is "not x"); macro invocation (`println!`); in Rust, and rarely, the *never* type — `fn abort() -> !` declares a function that does not return, and neither repository uses it; and, in Niles only, the effect row (`-> T ! { append }`).

## 32. `unsafe`, and its absence

```rust
unsafe impl GlobalAlloc for Counting { ... }
```

`unsafe` is the door out of Rust's guarantees. Inside such a block you may do things the compiler cannot verify — and you are promising you have verified them yourself.

In this project the door is opened in exactly one place: a tool that counts memory allocations. There are two copies of it, one per repository, and each is excluded from its own workspace — so a command that says "build everything" never reaches either. Inside, four items are marked `unsafe`, because replacing a program's allocator cannot be written any other way.

Nothing that ships links to it, and two tests — one in each repository, both named *the only unsafe in the repository is the measurement tool* — fail if that stops being true. Both build their search string in two halves at run time, so that the test does not match its own source and pass by accident.

Niles reserves the word and rejects it outright. The compiler's own table gives the reason in six words: there is no unsafe fragment of Niles.

---

# Part IV — The engines' own vocabulary

These are not keywords of any language. They are names this project invented, and you will meet them on nearly every page of the two repositories. Knowing what twenty of them mean is most of what it takes to read the code.

## 33. Nilestream — the database engine

### The write path (`nilestream-ledger`)

| Name | What it is | The metaphor |
|---|---|---|
| `Epoch(u64)` | A tick of the system clock: one sealed batch of writes. A counter, not a time. (The ledger crate wraps it in its own type; the read-path crate spells the same idea `type Epoch = u64`.) | The page number of the vault book. |
| `Minor = i128` | An amount, as an exact whole number of minor units. | Counting in cents, never in dollars-and-a-fraction. |
| `Txn` | A transaction: the group of movements the conservation rule is checked over. | One complete entry in the book, which must balance. |
| `Record` | One row as it is stored. | One line. |
| `Segment` | A chunk of the ledger on disk. | One physical volume of the book. |
| `Sequencer` | The component that puts writes in order and seals epochs. | The single desk where the current page is written. |
| `Frontier` | The newest epoch that is visible to readers. | How far the book has been sealed. |
| `Hasher256`, `chain_hash` | The fingerprint machinery: a 32-byte SHA-256 seal computed over an epoch's rows, its number, and the previous epoch's seal (`chain_hash(parent, body)`). | The wax seal that makes tampering detectable. |
| `Commitment` | Not the chain seal, despite the name. A 32-byte `H(salt ‖ value)` kept in the confidentiality sidecar in place of a value, so the chain can commit to a figure it cannot read. | The sealed envelope's fingerprint — what the book records instead of the contents. |
| `Snapshot`, `Recovery` | Restart machinery: what was saved, and how the engine returns after a crash. | Where the bookmark was when the lights went out. |
| `SyncPolicy` | How hard the engine insists the disk really wrote it. | Whether you watch the postbox door close. |
| `TruncationCause` | Why the tail of the log was found incomplete. | The half-written last line after a power cut. |

### The read path (`nilestream-core`)

| Name | What it is | The metaphor |
|---|---|---|
| `Rev` | The **reconstructible epoch-anchored view** — the thesis's central object. A derived answer-set that keeps only what is asked for and can rebuild the rest. | The clerk's notebook. |
| `Key = Vec<i64>` | Which group an answer belongs to — account and currency, say. | Which line of the notebook. |
| `Value = i128` | The answer. | The figure written there. |
| `Slot<V>` | What is in a notebook line, in four states: `Bottom` (never asked), `Hole` (computed and later erased, remembering when it was last right), `Pending` (being fetched now), `Present` (the value and its epoch). | Blank-because-nobody-asked, blank-because-we-rubbed-it-out, blank-because-the-clerk-is-still-walking, and written. |
| `Anchored` | A value *and* the epoch it is true at. The return type of every read. | An answer with its "correct as of page N" stamp. |
| `Base` | The trait a source must implement to be foldable: which questions it answers, how far it has got, how to rebuild a key at an epoch, one epoch's changes, and how many rows that epoch touched. | The vault book, described by what a clerk may ask of it. |
| `BasePlan` | What a given base is permitted to answer. | The index card saying which questions this book can settle. |
| `ReadOutcome` | `Hit` — resident, here it is; `Fold` — not resident, go and rebuild it; `Join` — someone is already rebuilding this key at this exact epoch, wait for theirs. | "It's in my notebook", "I'm walking to the vault", and "she's already gone for exactly that page". |
| `FoldTicket`, `WaitTicket` | A claim on a reconstruction you are performing, and on one you are waiting for. A ticket cleans up after itself if abandoned. | The numbered stub you are handed while the clerk is away. |
| `ReadMode` | Whether this reader folds alone or may join another's trip. | Going yourself versus asking someone already going. |
| `MergeCaps` | Ceilings on how much a deferred merge may fold before refusing. | How many pages the clerk will read before saying "come back later". |
| `Policy` | The eviction rule: which line gets rubbed out when the notebook is full. | Which entry the clerk sacrifices for space. |
| `Stats` | The counters an operator watches. | The tally sheet on the wall. |
| `Runtime` | The thing that holds every view and answers reads. | The whole clerks' room. |
| `Shard`, `ShardMap`, `Cluster` | Splitting the work across machines. Designed, not built. | More rooms, in more buildings. |

### The language and its intermediate form

| Name | What it is |
|---|---|
| `Circuit` | The IR: the wall of flow charts from Appendix A.8, as data. |
| `Node`, `Op` | One box on a chart, and what kind of box it is. |
| `Agg`, `Scalar` | An aggregation (`sum`, `count`) and a per-value computation. |
| `ZSet = BTreeMap<..>` | A set in which membership has a *weight*, so "add this row" and "remove this row" are the same operation with opposite signs. This is the mathematics that makes nudging work. |
| `ServeContract` | The `serve { ... }` block as data: `Consistency`, `Materialize`, `Retention`, `Lineage`. |
| `UpqueryPath`, `Hop`, `Step` | The route backwards from a question to the ledger pages that answer it — Appendix A.10, as a data structure. |
| `VerifyReport`, `Violation` | What the independent IR checker found. Deliberately a second implementation of rules the compiler already enforces, so that a mistake must be made twice to escape. |
| `Tok`, `Kw`, `Item`, `Expr`, `Pat`, `Ty` | The compiler's own stages: tokens, keywords, declarations, expressions, patterns, types (§5). |

## 34. GBS — the core-banking platform

GBS is a separate artifact: a complete banking platform — accounts, entries, holds, schedules, foreign exchange, lending, trade finance, securities, liquidity — with no external dependencies at all, and one excluded crate that connects it to Nilestream.

| Name | What it is |
|---|---|
| `Account`, `AccountId` | An account, and its identifier as a distinct type (§25). |
| `Amount { minor, currency, scale }` | Money as three inseparable parts. |
| `Entry`, `Side`, `Nature` | One line of double-entry: the movement, which side (debit or credit), and what kind of account it belongs to. |
| `Chart` | The chart of accounts: the tree every entry must land in. |
| `PostingSet`, `Sealed` | A set of entries as assembled, and one that has been checked and sealed. The type says which, so an unchecked set cannot be filed by mistake — this is §25's tuple-struct discipline applied to the thing that matters most. |
| `Committed` | Unrelated to `Sealed` despite the name: the narrative of a committed-confidentiality encoding. |
| `LedgerSink` | The trait that says "somewhere sealed entries can be sent". `MemoryLedger` implements it; so does the Nilestream connector. This is the seam between the two projects. |
| `AnchorIndex` | GBS's own version of the mandatory ledger index. |
| `Confidential`, `Reading` | The encrypted-field machinery and what a decrypted look at it produces. |
| `KeyEvent`, `ErasureEvent` | Key rotation, and the crypto-shredding that stands in for erasure in a book that cannot be erased (Appendix A.30, case 2). |
| `KernelError`, `LedgerError`, `KeyError` | The failure kinds, each an enum, so every caller must decide what to do about each. |
| `gbs-directory` | The one crate that genuinely deletes. Identity data lives here precisely *because* it must be erasable, and it is kept out of the ledger for that reason. |

---

# Part V — The complete lookup

## 35. Every Niles keyword, A to Z

All 174 words the compiler knows, in one alphabetical list. The **From** column says where the word comes from: **SQL**, **Rust**, **new** (invented for Niles), or **reserved** (set aside, with no meaning yet — using one is a hard error naming the reason).

A note before the table. Being a keyword in Niles mostly does *not* stop you using the word as a name for something of your own. Niles deliberately keeps words like `epoch`, `ledger`, `serve` and `budget` available as column names, because a bank's existing schema is not going to be renamed to suit a new language. Every invented word in this list is *unreserved* for that reason, and a test enforces it. Where a word genuinely cannot be a name, `r#word` — with a hash in front — makes it one anyway.

| Keyword | From | What it does | The picture |
|---|---|---|---|
| `absent` | new | Materialization mode: keep nothing resident; every read rebuilds from the ledger. | A clerk with no notebook at all, who goes to the vault every time. |
| `actor` | reserved | Set aside. No meaning assigned. | A word held back in case it is needed. |
| `add` | SQL | Add a column or constraint to an existing table. | Ruling a new column onto the form. |
| `all` | SQL | Keep duplicates in a set operation; also the universal quantifier ("for every"). | "Everything, including the repeats." |
| `alter` | SQL | Change a declared object. Legal on a `table`; never on a `base` or `ledger`. | You may redesign the whiteboard. You may not redesign the bound book. |
| `anchor` | new | The epoch stamp every answer carries, and the mandatory index kind on a ledger key. | "Correct as of page 41,209" — and the tabbed divider that lets you find page 41,209. |
| `and` | SQL | Both conditions must hold. Stops early if the first fails. | Two boxes that must both be ticked; no point reading the second if the first is blank. |
| `any` | SQL | At least one of a collection satisfies the condition. | "Is there anybody here who…?" |
| `as` | SQL | Rename a column, label an output, or convert a value's type. | "…hereinafter referred to as…" |
| `as_of` | new | Pin a read to a point on the *system* calendar: what we knew then. | "Never mind today — what did the file say on Wednesday?" |
| `asc` | SQL | Sort ascending. The default. | A to Z. |
| `async` | reserved | Set aside. Query and transaction context is synchronous by design. | A door deliberately not fitted. |
| `authorize` | new | Check a floor against a capability. **The only construct that may overdraw.** | The one counter with a manager's key, and no other way past the limit. |
| `auto` | new | Let the optimizer choose how much to keep, within the rest of the contract. | Delegating to the head clerk — who, today, does not exist. |
| `await` | reserved | Set aside, with `async`. | — |
| `backfill` | new | Populate a newly declared view from history, without re-deriving the base. | Opening a new notebook and filling it in from the book, back to a chosen page. |
| `base` | new | An immutable, fully retained, epoch-ordered relation. The authoritative kind. | The bound book with no eraser. |
| `begin` | SQL | Open a *table* transaction. Ledger writes use `txn` instead. | Opening a working session on the whiteboard. |
| `between` | SQL | Inclusive range test: `x between (lo, hi)`. | "Somewhere between these two marks, ends included." |
| `bitemporal` | new | Declare both calendars on a relation: when recorded, and when true. | Two date stamps on every line. |
| `bounded` | new | Consistency rung 0: no more than K epochs or T milliseconds behind. | "A few pages out of date at most, and never more than five seconds old." |
| `break` | Rust | Leave a loop early. Illegal inside a guarded recursion, which must end by measure. | Walking out of the room mid-task. |
| `budget` | new | The ceiling on a view's resident state, in entries or bytes. | How many pages the notebook has. |
| `by` | SQL | Introduces a grouping or ordering key. Never stands alone. | The "…by…" in "sorted by surname". |
| `capability` | new | Declare an unforgeable token authorising a specific effect. | A signed warrant that cannot be photocopied. |
| `case` | SQL | Multi-way conditional expression. Rust's `match` is the alternative and is exhaustive; this is not. | A list of "if this, then that" lines with an "otherwise". |
| `cast` | SQL | Explicit conversion. **Never crosses a currency.** | Converting inches to centimetres. There is no such conversion from dollars to euros. |
| `check` | SQL | A row-level constraint on a table. | A rule printed on the form that every entry must satisfy. |
| `column` | SQL | Names a column in a declaration. | The heading of one column of the form. |
| `commit` | SQL | Close a table transaction; its writes become visible at the next epoch. | Handing the work in. |
| `committed` | new | Confidentiality level: an additively homomorphic commitment, usable in sum-checks only. **Not implemented**: no commitment scheme exists, and `NL0260` refuses a `sum` over such a column exactly as it would over an `e2ee` one, so the two levels are the same level in the compiler (Section 17). | An envelope you can weigh without unsealing — and a stack of them weighs exactly what the amounts inside add up to. |
| `confidential` | new | Mark a column encrypted; the engine may not compute on it. | A sealed envelope filed by its label only. |
| `conserve` | new | **The double-entry rule.** The group's amounts must sum to zero, per currency. | Every entry balances, or it is not an entry. |
| `consistency` | new | Which rung of the freshness ladder a view is served at. | How recent an answer has to be before you will accept it. |
| `const` | Rust | A compile-time constant. Must be pure and epoch-independent. | A figure printed on the form rather than filled in. |
| `continue` | Rust | Skip to the next iteration of a loop. | "Next." |
| `crate` | Rust | The root of the current compilation unit, in a path. | The top of this filing cabinet. |
| `create` | SQL | Declare an object. Optional inside a `schema` block. | Opening a new file. |
| `cross` | SQL | Unrestricted product join: every row with every row. | Every name on list A paired with every name on list B. |
| `currency` | new | Declare a currency and its minor-unit scale. Scale lives in the type. | Declaring that this jar holds hundredths and that one holds whole units. |
| `declassify` | new | The single audited construct that lowers a confidentiality level. | The one authorised letter-opener, and it keeps a log. |
| `default` | SQL | A column's default value. Must be pure and epoch-independent. | What goes in the box when it is left blank. |
| `delete` | SQL | Remove rows. **`table` only** — a ledger has no delete. | Wiping the whiteboard. Not available for the book. |
| `demand` | new | Materialization mode: keep only what is read; evict the rest; rebuild on demand. | The notebook this whole thesis is about. |
| `desc` | SQL | Sort descending. | Z to A. |
| `distinct` | SQL | Remove duplicates. | One line per unique value. |
| `drop` | SQL | Remove a declared object. **Never removes ledger history.** | Throwing away a form's design, not the forms already filed. |
| `dyn` | Rust | Decide which implementation to use while running. Forbidden in view bodies. | Asking at the door which department handles this. |
| `e2ee` | new | Confidentiality level: end-to-end encrypted, opaque to the engine. | A sealed envelope; only the customer has the key. |
| `else` | SQL | The alternative branch of a conditional. | "Otherwise…" |
| `emit` | new | Publish a derived change downstream as a stream. | Posting a copy of each update to whoever subscribed. |
| `end` | SQL | Closes a `case` expression. | The full stop on a list of conditions. |
| `enum` | Rust | A type that is exactly one of several named cases, each able to carry data. | Tick-boxes where exactly one must be ticked, each with its own follow-up. |
| `epoch` | new | The unit of visibility, versioning and hashing. Written `#41209`. | The page number of the vault book. |
| `evictable` | new | Retention: this derived state may be dropped and rebuilt. | A page that may be rubbed out for space. |
| `except` | SQL | Set difference: in the first, not in the second. | "These, minus those." |
| `exists` | SQL | Is there at least one row? | "Is the drawer empty?" |
| `expire` | new | Resolve a hold by lapse of its window, releasing the reservation. | The set-aside money goes back because nobody claimed it in time. |
| `expires` | new | The window on a hold or an idempotency key. | The date stamped on the reservation slip. |
| `explain` | new | Show how an answer was derived, at the view's lineage mode. | "Show your working." |
| `false` | Rust | The boolean literal. | An unticked box. |
| `fn` | Rust | Declare a function. **Its effect row is part of its type.** | A named procedure whose cover sheet lists the departments it touches. |
| `for` | Rust | Iterate over a collection; also the head of `impl Trait for Type`. | "For each of these, do the following." |
| `foreign` | SQL | Introduces a foreign-key constraint. | "This number must exist on that other form." |
| `forever` | new | Retention: never evicted. **Mandatory on a base or ledger.** | The book is kept. That is not negotiable. |
| `freshness` | new | The staleness bound of a `bounded` rung, as a duration. | "No older than thirty seconds." |
| `from` | SQL | The source relation. | "…taken from the postings book." |
| `full` | SQL | Full outer join; also the `full` materialization mode and `lineage: full`. | Keep everything on both sides, matched or not. |
| `fx` | new | Atomic cross-currency form: two conserved legs sealed in one epoch. | Two balanced moves, one per jar, tied together by one order. |
| `grant` | SQL | Confer a capability. Niles grants a token, not an ambient role. | Issuing the warrant. |
| `group` | SQL | Introduces grouping. The pipeline spelling is `group_by`. | Sorting the forms into piles. |
| `guard` | new | Attach the termination witness to a recursion. Unguarded recursion is rejected. | The written promise that the count is going down and will stop. |
| `having` | SQL | Filter applied *after* grouping, over group totals. | "Keep only the piles that add to more than a thousand." |
| `hold` | new | Reserve funds as a ledger fact. **Linear: resolvable exactly once.** | Money set aside on the counter — spoken for, not yet moved. |
| `idem` | new | The idempotency key: a name for an instruction, so a repeat is recognised. | A reference number on a cheque; the second presentation is refused. |
| `if` | Rust | A conditional. In Rust and Niles it is an expression: it produces a value. | "If this, then that; otherwise the other." |
| `impact` | new | The inverse of `explain`: which views a base row can affect. | "This entry was wrong. What did it touch?" |
| `impl` | Rust | Attach functions to a type, or declare that a type has a trait. | Filling in what this thing can do. |
| `in` | SQL | Membership test, and the binder in `for x in xs`. | "Is this name on the list?" |
| `index` | SQL | Declare an index. Anchor indices are **mandatory** on ledger keys. | Tabbed dividers in the binder. |
| `inner` | SQL | Inner join. The default: keep only rows that match on both sides. | Only the pairs where both halves were found. |
| `insert` | SQL | Add rows to a `table`. A ledger is appended to with `txn`, not this. | Writing on the whiteboard. |
| `intersect` | SQL | Set intersection: in both. | "Only the names on both lists." |
| `into` | SQL | The target of an `insert`. | "…into this drawer." |
| `is` | SQL | Identity test, chiefly `is null` / `is not null`. | "Is this box actually blank?" |
| `join` | SQL | Combine two relations on a condition. | Matching two lists by a shared column. |
| `key` | SQL | Part of `primary key` / `foreign key`; also `lineage: key`. | The column that identifies a row uniquely. |
| `ledger` | new | A `base` plus a conservation rule and typed money. **Never partial.** | The vault book itself. |
| `ledger_consistent` | new | Consistency rung 5: anchored at the visibility frontier exactly. The authorization path. | "This very second, no exceptions." |
| `left` | SQL | Left outer join: keep all of the left side. | Keep every name on the first list, matched or not. |
| `leg` | new | One side of an `fx` form. | One of the two balanced moves. |
| `let` | Rust | Bind a value to a name. Linear values bound here must be consumed exactly once. | "Let the rent be 850 dollars." |
| `like` | SQL | Pattern match on text. | "Anything starting with 'Mc'." |
| `limit` | SQL | Cap how many rows come back. | "The first fifty only." |
| `lineage` | new | Provenance mode: `off`, `key` or `full`. Even `off` still stamps the anchor. | How much of the working you keep alongside the answer. |
| `loop` | Rust | Loop unconditionally. Not permitted in a view body. | "Keep going." |
| `macro` | reserved | Set aside. | — |
| `match` | Rust | Pattern-match a value. **Exhaustive**, unlike `case`. | A decision table the auditor rejects until every row is filled. |
| `materialize` | new | The mode a view's state is kept in. | How much of the notebook is actually written down. |
| `measure` | new | The well-founded quantity a guarded recursion must decrease. | The number that counts down, proving it stops. |
| `mod` | Rust | A module: a named drawer of items. | A drawer in the cabinet. |
| `monotonic` | new | Consistency rung 1: within a session, anchors never decrease. | Answers never step backwards in time. |
| `move` | Rust | A closure captures by value rather than by reference. Required across an epoch boundary. | Taking the papers with you rather than pointing at them. |
| `mut` | Rust | A binding may be changed. **Never applies to a sealed ledger row.** | Asking for a pencil instead of a pen. |
| `natural` | SQL | Join on all like-named columns. Discouraged: it breaks when a schema changes. | Matching two forms by "whatever headings happen to agree". |
| `not` | SQL | Boolean negation. | "…and not this one." |
| `null` | SQL | SQL's null. Distinct from Rust's `None` *and* from an evicted hole. | A blank box — which is not the same as a box we erased. |
| `of` | new | Connective in `as_of` and `impact of`. | The "of" in "as of Wednesday". |
| `offset` | SQL | Skip a number of rows before returning any. | "Start from the fifty-first." |
| `on` | SQL | The join predicate, or the object of a `grant`. | "…matched on account number." |
| `or` | SQL | Either condition. Stops early if the first holds. | Two boxes, either will do. |
| `order` | SQL | Introduces ordering. The pipeline spelling is `order_by`. | "Sorted by…" |
| `outer` | SQL | Marks a join as outer: keep unmatched rows. | Keep the lonely entries too. |
| `per` | new | Introduces the grouping of a conservation rule: `conserve per (txn, cur)`. | "Balance within each transaction, and within each currency separately." |
| `pinned` | new | Retention: resident and never evicted, at a declared memory cost. | A page nailed to the desk. |
| `post` | new | Resolve a hold into a posting, capturing up to the held amount. | Turning the set-aside money into an actual movement. |
| `posting` | new | One signed movement; the linear unit a ledger row is built from. | One half of a double entry. |
| `primary` | SQL | Introduces the primary key. | The column that names the row. |
| `pub` | Rust | Export from a module or schema. Private is the default. | Putting it on the counter rather than in the back office. |
| `rate` | new | The declared conversion of an `fx` form, recorded with the legs. | The exchange rate written on the order. |
| `read_your_writes` | new | Consistency rung 2: a session sees at least its own committed writes. | "You just paid in; of course you can see it." |
| `recorded_at` | new | The system-time axis: when the fact was written down. **Never rewritten.** | The date stamp the clerk applies. |
| `recursive` | SQL | Marks a query as referring to itself. Niles requires a `guard measure(..)` regardless. | A rule that applies to its own output — with a proof that it stops. |
| `ref` | Rust | Bind by reference in a pattern. | Looking at it without taking it. |
| `references` | SQL | The target of a foreign key. | "…refers to the accounts form." |
| `reproduce` | new | Re-derive an answer from the base and compare, as an audit act. | Recounting from the book to see if the notebook was right. |
| `resolve` | new | Consume a hold exactly once, by `post`, `void` or `expire`. | Closing out the reservation. It must end exactly one of three ways. |
| `retain` | new | Retention of derived state. The base is always retained. | Whether this page may be rubbed out. |
| `return` | Rust | Return early from a function. | Handing the file back before reaching the end. |
| `revoke` | SQL | Withdraw a capability. **Recorded as a ledger event, never a silent edit.** | Cancelling the warrant, in writing, in the book. |
| `right` | SQL | Right outer join. | Keep every name on the second list. |
| `rollback` | SQL | Abandon a table transaction. A sealed ledger epoch cannot be rolled back. | Wiping the working session. The book is untouched by this word. |
| `scale` | new | A currency's minor-unit exponent. Part of the type, never assumed. | How many decimal places this currency actually has. |
| `schema` | new | The declaration unit: currencies, tables, bases, ledgers, views, indices. | The chapter of the manual that says what exists. |
| `select` | SQL | Projection: choose the columns. Pipeline spelling is `.map` / `.select`. | "Just these columns, please." |
| `Self` | Rust | The type currently being implemented. | "…this kind of thing." |
| `self` | Rust | The receiver parameter, or a path root. | "…this particular one." |
| `serializable` | new | Consistency rung 4: reads and writes form one serial order. | Everything that happened can be put in one queue that explains it. |
| `serve` | new | Attach a contract to a view. **The contract is part of the view's type.** | The service promise printed on the notebook's cover. |
| `set` | SQL | The assignment list of an `update`. | "…set the address to…" |
| `signal` | new | An epoch-indexed value: balance as a function of time. | A film rather than a photograph. |
| `snapshot` | new | Consistency rung 3: one anchor for the whole read, across views. | Every figure in this report is from the same moment. |
| `spilled` | new | Materialization mode: resident on slower storage. **Illegal at rung 5.** | The notebook is on the shelf, not the desk. |
| `sql` | new | Escape into the SQL surface. **Same IR, different syntax.** | A paragraph written in the old language, clearly marked. |
| `static` | Rust | A program-lifetime item. Immutable; there is no mutable static. | A notice on the wall for the whole working day. |
| `stream` | reserved | Set aside. | — |
| `struct` | Rust | A type with named parts. | A form with labelled boxes. |
| `super` | Rust | The parent module in a path. | The drawer above this one. |
| `table` | SQL | A mutable relation: `update` and `delete` are legal, history is not kept. | The whiteboard. |
| `then` | SQL | The consequent of a `case` arm. | "…then this." |
| `tiered` | new | Materialization mode: hot resident, cold spilled, decided by the optimizer. | Some pages on the desk, the rest on the shelf. |
| `trait` | Rust | An interface: a named set of capabilities a type may claim. | A professional qualification, not a job title. |
| `true` | Rust | The boolean literal. | A ticked box. |
| `txn` | new | A ledger transaction: the unit the conservation rule is checked over. | One complete entry, which must balance before it is accepted. |
| `type` | Rust | A type alias or an associated type. | A second name for a kind of thing. |
| `udf` | new | A user-defined function: fuel-metered, sandboxed, compiled to WASM. | Custom logic in a locked room with a work allowance and no clock. |
| `union` | SQL | Set union, removing duplicates. | Both lists merged, each name once. |
| `unique` | SQL | A uniqueness constraint. | "No two rows may share this." |
| `unsafe` | reserved | Set aside and rejected: **there is no unsafe fragment of Niles.** | A door bricked up rather than locked. |
| `update` | SQL | Modify rows. **`table` only** — a ledger has no update. | Rewriting on the whiteboard. |
| `upto` | new | Bound a backfill or a replay at an epoch. | "Fill in the notebook as far as page 41,209 and stop." |
| `use` | Rust | Import a path so a name can be written short. | Keeping the file you need on the desk. |
| `using` | SQL | Join on named common columns. | "Matched on account number and currency." |
| `valid_at` | new | The valid-time axis: when the fact was true in the world. | The date the money actually moved. |
| `value_date` | new | The banking value date of a posting; drives valid-time placement. | The day this counts for interest. |
| `values` | SQL | A literal row constructor in an `insert`. | The contents of a row, written out. |
| `view` | SQL | A derived relation with a serve contract. **The REV of the theory.** | The clerk's notebook, with its promise on the cover. |
| `void` | new | Resolve a hold by cancelling it: release the reservation, post nothing. | The reservation is torn up; the money was never spent. |
| `when` | SQL | The guard of a `case` or `match` arm. | "…in the case where…" |
| `where` | SQL | The filter stage, and Rust's bound clause. The positions are disjoint. | "Keep only the ones where…" |
| `while` | Rust | A conditional loop. Not permitted in a view body. | "Keep going as long as…" |
| `window` | new | How long an idempotency key is remembered, in epochs. | How long the clerk keeps the list of cheque numbers already seen. |
| `with` | SQL | A named sub-query in the SQL surface. | "Given the following working definition…" |
| `yield` | reserved | Set aside. | — |

## 36. Every Niles sign

| Sign | Read it as | What it does | The picture |
|---|---|---|---|
| `;` | full stop | Ends a statement. | The end of the sentence. |
| `:` | "of type" | Separates a name from its type: `amt: Money`. | The heading above the box, saying what goes in it. |
| `,` | and | Separates items in a list. | The comma on a form. |
| `.` | "'s" | Access a field or apply a step: `p.amt`. | The apostrophe-s in "the posting's amount". |
| `::` | path separator | Navigate into a module or type: `std::bank::fx`. | The slashes in a folder path. |
| `()` | grouping / call | Group an expression, or call a function. | Brackets, and the act of asking. |
| `{}` | a block | Groups statements, or a struct's contents. | A section of the document. |
| `[]` | index / slice | Pick out an element or a range. | "The third one." |
| `<>` | type parameters | `Money<usd>`, `Option<T>`. | The blank on the form, filled in. |
| `->` | "produces" | A function's return type. | The arrow on the flowchart to the outcome. |
| `=>` | "then" | Separates a pattern from its consequence in `match`. | The arrow in a decision table. |
| `=` | "becomes" | Assignment. Binds loosest of all, and groups to the right. | Writing a value into a box. |
| `==` `!=` | equals / not equals | Comparison. Two signs, because one means *assign*. | "Is it the same?" versus "make it the same." |
| `<` `<=` `>` `>=` | ordering | Comparison. | Less, at most, more, at least. |
| `+` `-` `*` `/` `%` | arithmetic | Add, subtract, multiply, divide, remainder. | Ordinary arithmetic. `%` is the remainder after division. |
| `&&` `\|\|` `!` | and / or / not | Boolean logic. `and`, `or`, `not` are the same operators spelled as words, and produce the identical result. | Ticking, either-or, and the negation. |
| `\|x\| ...` | "given x" | A closure: a small anonymous function. | An instruction slip with a blank, and `x` names the blank. |
| `\|>` | "then" | An explicit pipeline stage. | The arrow between steps of a process. |
| `?` | "or give up" | If this failed, return the failure to the caller. | "…and if refused, hand the file back with the reason." |
| `!` after a signature | "with effects" | Introduces the effect row: `-> T ! { append }`. | The cover sheet listing the departments this touches. |
| `@` | "at" | Introduces a temporal literal (`@2026-08-25`) and annotations (`@confidential`). | A date stamp, and a marginal note. |
| `v@` | "value-dated" | A value-date literal specifically. | The banking date, not the calendar date. |
| `#` | "page" | Introduces an epoch literal: `#41209`. | The page number. |
| `#[...]` | an attribute | A note attached to the item below: `#[udf(fuel = ...)]`. | A sticker on the folder. |
| `_` | "don't care" | A wildcard in a pattern; also a digit separator in numbers. | The "any" box, and the space in 1 000 000. |
| `..` `..=` | a range | Up to, and up to *and including*. | "Pages 1 to 10" — with and without page 10. |
| `//` `/* */` `///` `//!` | a comment | Ignored by the compiler. `///` and `//!` are documentation. | A note in the margin. |
| `--` | a comment, SQL-style | Accepted **only inside `sql { }`**. | A margin note in the old language. |
| `r#name` | an escape | Use a reserved word as an ordinary name. | Quotation marks around a word being used as a name. |

## 37. Every Niles literal

| Written | Is | Note |
|---|---|---|
| `42`, `1_000_000` | Whole numbers, in base 10. | `_` between digits is a separator the compiler ignores. |
| `3.14` | An approximate fractional number. | **Banned in money positions by type**, not by convention. |
| `"text"`, `b"bytes"` | Text with Rust's escapes, and raw bytes. | |
| `true`, `false` | Booleans. | |
| `()` | Unit: no information. | What a function returns when it returns nothing. |
| `125.00 usd`, `1_000 jpy`, `12.345 kwd` | **Money.** Number and currency as one token. | Fractional digits must match the declared scale exactly. The grade must be exactly three lowercase letters — the limitation of Appendix A.29. |
| `@2026-08-25T12:00:00Z` | An instant. | |
| `@2026-08-25` | A date. | |
| `v@2026-08-25` | A value date. | The banking axis, distinct from the calendar one. |
| `#41209` | An epoch. | A page of the vault book. |
| `3.epochs`, `250.ms`, `2.days` | Durations. | |
| `@2026-01-01 ..= @2026-12-31` | A closed valid-time interval. | |
| `idem("payment-7f3a")` | An idempotency key. | The reference number on the cheque. |

**Nine forms that thesis Appendix B.2 lists and the compiler does not have.** This table was checked by feeding each form to `nilesc check` rather than by copying the appendix, and the appendix overclaims. Hexadecimal, octal and binary integers (`0xFF`, `0o77`, `0b1010`), exponent floats (`6.02e23`), typed numeric suffixes (`42i64`, `7u32`) and raw strings (`r"…"`, `r#"…"#`) all fail at the lexer today — each is read as a number or an identifier followed by something unexpected, and rejected with `NL0001`. So does `idem"payment-7f3a"`, the idempotency key written directly against a string: the parser reads `idem` and then requires an argument list, so the form that exists is `idem("payment-7f3a")`, which is what `examples/demo_bank.niles` uses and what §15 and §41 of this guide show. `examples/available_balance.niles` already names that last spelling as one of the constructs the language does not have. In every case the appendix is describing an intention and the lexer is describing the language; this guide lists what lexes.

The money rules, by contrast, are exactly as described and each has its own diagnostic: `2.5 usd` is refused as `NL0240` ("this literal has 1 decimal places but `usd` has scale 2"), and so is `1_000.00 jpy`; `12.345 kwd` and `1_000 jpy` are accepted.

## 38. Every Niles type

**Primitives.** `bool`; `i8` through `i128` and `u8` through `u128` (whole numbers, signed and unsigned, of that many bits); `f32`, `f64` (approximate fractions, never money); `Text`; `Bytes`; `()`.

**Composites.** Tuples `(a, b)`; arrays and slices; `Vec<T>` (a growable list); `Option<T>` (present or absent); `Result<T, E>` (succeeded or failed); structs; enums; references; `Json` for semi-structured columns.

**The invented ones** — this is where the language earns its existence:

| Type | Means | The picture |
|---|---|---|
| `Money<CUR>` | An exact whole number of minor units in currency `CUR`. Closed under `+`, `−` and integer scaling. **No operation joins two currencies.** | Coins in a labelled jar. You cannot add the dollar jar to the euro jar; the arithmetic does not exist. |
| `Currency` | A currency, declared with its scale. | The label on the jar, including how much a coin is worth. |
| `Id<T>` | An identifier for a `T` specifically. | An account number that cannot be mistaken for an order number. |
| `Epoch`, `Instant`, `Date`, `Duration`, `Interval<Valid>` | The temporal kinds. | Page numbers, moments, days, spans, and stretches of world-time. |
| `Signal<T>` | A value as a function of epoch. | The film, not the photograph. |
| `Bitemporal<T>` | A value on both calendars. | A fact with both date stamps. |
| `Posting`, `Debit<CUR>`, `Credit<CUR>` | Linear halves of a movement, consumed **exactly once**. | A banknote: not copyable, and it cannot be left on the table. |
| `Hold<CUR>` | A reservation, resolvable exactly once by post, void or expire. | The slip on the set-aside cash, which must be torn up or cashed. |
| `Auth<E>` | An unforgeable capability for effect `E`. | The signed warrant. |
| `Anchored<T>` | A value with its epoch. **The return type of every view read.** | The answer with its "as of page N" stamp attached. |
| `IdemKey` | An idempotency key with a declared window. | The cheque reference and how long it is remembered. |
| `Lineage<T>` | A provenance annotation at the view's declared mode. | The working, kept alongside the answer. |

**Effect rows** are the last piece: `fn(..) -> T ! {read@snapshot, append, debit<usd>}`. The part after `!` is *part of the type*, so what a function does to the world is visible in its signature and checked at every call.

---

## 39. Every Rust keyword, A to Z

Rust's full reserved vocabulary. The **In use** column says whether the word appears in this project's shipping code — several do not, and the reasons are interesting.

| Keyword | In use | What it does | The picture |
|---|---|---|---|
| `abstract` | no (reserved) | Reserved, unused by the language. | — |
| `as` | yes | Convert a type (`x as u64`), or rename an import (`use a::b as c`). | "…counted as…", and "…hereinafter called…". |
| `async` | **no — by design** | Mark a function as one that can pause and resume. | A task that can be set aside and picked up. This project's engine is deliberately synchronous. |
| `await` | **no — by design** | Wait for an async task. | — |
| `become` | no (reserved) | Reserved for a future tail-call feature. | — |
| `box` | no (reserved) | Reserved; heap allocation is `Box::new` instead. | — |
| `break` | yes | Leave a loop, optionally with a value. | Walking out. |
| `const` | yes | A compile-time constant, or a function that can run at compile time. | A figure printed on the form. |
| `continue` | yes | Skip to the next iteration. | "Next." |
| `crate` | yes | The root of this compilation unit, in a path. | The top of this filing cabinet. |
| `do` | no (reserved) | Reserved. | — |
| `dyn` | yes (62×) | Choose the implementation while running. | Asking at the door which department handles this. |
| `else` | yes | The alternative branch; also `let ... else`. | "Otherwise…" |
| `enum` | yes | A type that is one of several named cases, each able to carry data. | Tick-boxes, exactly one ticked, each with a follow-up. |
| `extern` | no | Link to code written in another language. | A translated document from outside. Absent here: no foreign code. |
| `false` | yes | The boolean literal. | An unticked box. |
| `final` | no (reserved) | Reserved. | — |
| `fn` | yes (4,628×) | Declare a function. | A named procedure. |
| `for` | yes | Iterate; also the head of `impl Trait for Type`. | "For each…" |
| `if` | yes | A conditional expression. | "If… then… otherwise…" |
| `impl` | yes | Attach behaviour to a type, or declare it has a trait. Also `impl Trait` in a signature. | Filling in what this thing can do. |
| `in` | yes | The binder in `for x in xs`. | "…in this collection." |
| `let` | yes (10,678×) | Bind a value to a name. | "Let the budget be 100." |
| `loop` | yes | Loop forever until something breaks out. | "Keep going." |
| `macro` | no (reserved) | Reserved; `macro_rules!` is the form in use. | — |
| `macro_rules!` | yes (4 sites) | Define a macro. | Writing a rubber stamp. |
| `match` | yes (1,080×) | Pattern-match, exhaustively. | A decision table with no blank rows permitted. |
| `mod` | yes | Declare a module. | A drawer in the cabinet. |
| `move` | yes | A closure takes ownership of what it captures. | Taking the papers rather than pointing at them. |
| `mut` | yes (3,710×) | This binding, or this borrow, may change things. | Asking for a pencil. |
| `override` | no (reserved) | Reserved. | — |
| `priv` | no (reserved) | Reserved; private is already the default. | — |
| `pub` | yes | Make an item visible outside its module. | Putting it on the counter. |
| `ref` | yes | Bind by reference in a pattern. | Looking without taking. |
| `return` | yes | Return early. | Handing the file back. |
| `self` | yes | The receiver parameter, or a path root. | "…this particular one." |
| `Self` | yes | The type being implemented. | "…this kind of thing." |
| `static` | yes | A program-lifetime item at a fixed address. No mutable statics here. | A notice on the wall for the whole day. |
| `struct` | yes | A type with named parts. | A form with labelled boxes. |
| `super` | yes | The parent module. | The drawer above. |
| `trait` | yes (69×) | An interface: a named set of capabilities. | A professional qualification. |
| `true` | yes | The boolean literal. | A ticked box. |
| `type` | yes | A type alias or an associated type. | A second name for a kind of thing. |
| `typeof` | no (reserved) | Reserved. | — |
| `union` | no | A type whose fields share memory. Requires `unsafe` to read. Absent here. | One box, several conflicting labels. |
| `try` | no (reserved) | Reserved since the 2018 edition; both workspaces are 2021. | — |
| `unsafe` | **only in the two excluded measurement tools** | Step outside the compiler's guarantees. | The door out. Bricked up everywhere that ships. |
| `unsized` | no (reserved) | Reserved. | — |
| `use` | yes | Import a path. | Keeping a file on the desk. |
| `virtual` | no (reserved) | Reserved. | — |
| `where` | yes (686×) | Constrain a generic parameter. | The small print: "must be something countable." |
| `while` | yes | A conditional loop. | "As long as…" |
| `yield` | no (reserved) | Reserved. | — |
| `'static` | yes | A lifetime meaning "for the whole run of the program". | A loan with no due date, because it is never returned. |

## 40. Every Rust sign

Rust shares most punctuation with Niles (§36). These are the ones that are Rust's alone, or that mean something different here.

| Sign | Read it as | What it does | The picture |
|---|---|---|---|
| `&` | "a look at" | A shared reference: borrow to read. Any number at once. | The library's reference copy. Many may read it. |
| `&mut` | "exclusive use of" | An exclusive reference: borrow to change. Exactly one, and no readers meanwhile. | Taking the book away to write in it. |
| `*` | "the thing at" | Dereference: follow a reference to the value. Also multiplication. | Following the address to the actual house. |
| `'a` | "for as long as a" | A lifetime parameter. | The due date on the loan. |
| `::<>` | "specifically, as" | The turbofish: state a type the compiler could not infer. `parse::<u64>()` | "Read this as a number — that kind of number." |
| `!` after a name | a macro call | `println!`, `assert_eq!`, `vec![...]` | A rubber stamp rather than a procedure. |
| `!` before a value | "not" | Boolean negation, and bitwise complement. | The negation. |
| `#[...]` | an attribute | A note on the item below. | A sticker on the folder. |
| `#![...]` | an inner attribute | A note on the *enclosing* item, usually the whole file. | A sticker on the folder's cover, applying to everything in it. |
| `?` | "or return the error" | Unwrap a success, or return the failure to the caller. | "…and if refused, hand it back with the reason." |
| `->` | "produces" | A return type. | The outcome arrow. |
| `=>` | "then" | Pattern to consequence, in `match`. | The decision-table arrow. |
| `@` | "…and call it" | Bind a name while also matching a shape: `k @ (A \| B)`. | "Either of these two — and refer to it as k." |
| `_` | "don't care" | Wildcard pattern; also a digit separator. | The "any" box. |
| `..` | "and the rest" | A range, or "the remaining fields" in a pattern. | "…and so on." |
| `..=` | "up to and including" | An inclusive range. | The difference between ten and eleven items. |
| `\|` | "or" in a pattern | Alternatives in a pattern. Also bitwise or. | "Either of these shapes." |
| `\|x\| ...` | "given x" | A closure. | An instruction slip with a blank. |
| `::` | path separator | Navigate a module path. | The slashes in a folder path. |
| `dyn` / `impl` in a type | see §26 | Dynamic versus static dispatch. | Asking at the door versus knowing in advance. |
| `Vec<T>`, `&[T]` | list, window | An owned growable list; a borrowed view into part of one. | The whole folder; a few pages held open. |

---

# Part VI — Reading real files

## 41. A Niles file, line by line

From `examples/demo_bank.niles`. This is the thesis's worked example, and `every_niles_source_in_the_repository_compiles_and_verifies` in `crates/niles-lang/tests/niles_sources.rs` keeps it compiling, so it cannot drift away from the language it describes.

```niles
ledger postings {
    txn: TxnId,
    acct: Id<Account>,
    cur: Currency,
    amt: Money,
    value_date: Date,
    idem: IdemKey window 1_000_000.epochs,
    conserve per (txn, cur);
    retain forever;
    bitemporal;
}
```

*Declare an immutable, fully retained, conserved relation called `postings`. Each row has a transaction identifier, an account identifier, a currency, an amount, a value date, and an idempotency key remembered for a million epochs. Within any one transaction, and separately within any one currency, the amounts must sum to zero. This relation is never evicted. It keeps both calendars.*

Everything in it is a promise the compiler will enforce, not a comment.

```niles
view available_balance = postings
    .group_by(|p| (p.acct, p.cur))
    .sum(|p| p.amt)
    serve { consistency: ledger_consistent, materialize: demand, lineage: key };
```

*Define a standing question called `available_balance`. Take the postings; gather them into groups, where a row's group is the pair (its account, its currency); within each group add up the amounts. Serve this at the strictest freshness rung — anchored at the newest visible epoch — keeping resident only what has been read, and keeping key-level provenance.*

The comment above it in the real file is worth reading: this view originally derived from another view served three rungs looser — `read_your_writes` rather than `ledger_consistent` — and the compiler refused it, because staleness entering one step upstream cannot be removed by a downstream promise. That is a genuine banking failure — two views of one ledger disagreeing at the moment of a decision — caught by a typing rule on the thesis's own example.

```niles
fn card_authorization(acct: Id<Account>, amount: Money<usd>) -> Result<TxnId, TxnError>
    ! { append, hold<usd> }
{
    let h = hold(acct, amount, expires: 7.days)?;
    resolve h post 18.50 usd
}
```

*A function taking an account and a dollar amount. It produces either a transaction identifier or an error. Its effects, stated in its type, are: it appends to the ledger, and it places a dollar hold. Place a hold for that amount expiring in seven days — and if that fails, stop here and return the failure. Then resolve the hold by posting eighteen dollars fifty.*

Three things the type system is enforcing invisibly. `amount` is `Money<usd>`, so no euro amount can be passed. `h` is linear, so a version of this function that forgot the last line would not compile — a hold cannot be left unresolved. And `resolve` consumes it, so a version that resolved it twice would not compile either.

## 42. A Rust file, line by line

Three short excerpts, each from where it actually lives. The first is from `gbs/crates/gbs-kernel/src/ledger.rs`.

```rust
fn append(&mut self, set: &Sealed) -> Result<Epoch, LedgerError> {
    set.verify().map_err(LedgerError::Kernel)?;
    ...
    Ok(e)
}
```

*A function called `append`. It may modify the thing it belongs to (`&mut self`). It takes a look at a sealed set of entries without taking ownership of it (`&Sealed`). It produces either an epoch or a ledger error.*

*First, verify the set. If verification fails, convert the failure into this module's error kind and — the `?` — return it to the caller immediately. Otherwise carry on. At the end, produce the epoch, wrapped in `Ok` to say it succeeded.*

Every safety property in that paragraph is enforced. The caller keeps ownership of the set. The function cannot forget to handle a verification failure, because `verify` returns a `Result` and the only ways to proceed are to handle it or to forward it. And the type `Sealed` means an unsealed set cannot be passed at all: the check happened earlier, and the type is the evidence.

The second is from the other repository — `niles/crates/nilestream-core/src/absence.rs`, the engine's read path.

```rust
pub enum Slot<V> {
    Bottom,
    Hole(Epoch),
    Pending(Epoch),
    Present(V, Epoch),
}
```

*Declare, visible outside this module, a type called `Slot` that holds some type `V`. A value of this type is exactly one of those four cases.*

This is the type behind Appendix A.9. A notebook line that is blank because nobody asked is `Bottom`; one that is blank because it was computed and erased is a different case that remembers when it was last correct. Because these are distinct cases of one type rather than two meanings of one blank, every `match` in the engine is forced by the compiler to say what it does about each.

The third is a test, from `gbs/crates/gbs-kernel/src/entry.rs`.

```rust
#[test]
fn a_mirror_cannot_disagree_with_its_original() {
    ...
}
```

*Mark the following function as a test. It is called "a mirror cannot disagree with its original".*

A note on this project's convention, because it is unusual and it is load-bearing: tests are named as complete sentences stating the property being asserted. Not `test_mirror_1`. The name is the claim, so a test report reads as a list of the things the system promises — and a test whose name no longer describes what it checks is visible to anyone reading it.

---

## 43. Where to go next

- **`examples/demo_bank.niles`** — the complete worked Niles program, kept compiling by `tests/niles_sources.rs`.
- **`examples/inventory.niles`** — the same machinery counting warehouse stock rather than money. The falsifier of Appendix A.29.
- **`docs/keywords.md`** — the normative keyword reference, generated from the compiler's own registry. If this guide and that file ever disagree, that file is right: it cannot drift, because a test compares it to the lexer.
- **Thesis Appendix A** — the same ideas without any code at all.
- **Thesis Appendix B** — the formal syntax reference, for when you want the grammar rather than the explanation.

If something in the two repositories is not explained here, that is a gap in this guide. It was written to have none.

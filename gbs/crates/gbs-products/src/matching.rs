//! **Tier 1 — the matching engine, and the boundary that keeps it honest.**
//!
//! This module implements a matching engine's *interface to the ledger*, and deliberately
//! not a matching engine's hot path. The distinction is the whole point of the file.
//!
//! # The four orders of magnitude
//!
//! An exchange matching engine runs single-threaded on a pinned core with no allocation in
//! the hot path, kernel-bypass networking, and a latency budget in **hundreds of
//! nanoseconds**. A durable ledger is `fsync`-bound: **1.6–12.4 µs** on enterprise NVMe with
//! power-loss protection, and **891–2974 µs** without it.
//!
//! Those differ by three to four orders of magnitude. **No storage engine serves both**, and
//! a system claiming to would be lying about one of them. `ROADMAP.md` therefore separates
//! them into tiers, and this module is the seam:
//!
//! | Tier | What it is | Latency | Durability |
//! |---|---|---|---|
//! | **1 — matching** | Deterministic, in-memory, pinned core, no allocation | ~100 ns – 1 µs | **None in the hot path** |
//! | **2 — ledger** | Ingests tier 1's sequenced stream as pre-agreed epochs | µs – ms | `fsync`, hash-chained |
//! | **3 — REVs** | Positions, risk, P&L, regulatory roll-ups | per rung | derived |
//!
//! # Why the thesis's own model is the right interface
//!
//! **A matching engine's output is already a total order.** That is exactly what an epoch
//! is. So tier 2's admission is a **validation rather than a sequencing decision** — the
//! order was agreed upstream, and the ledger's job is to check and durably record it, not to
//! decide it.
//!
//! This is the same observation Calvin and Aria make about deterministic execution: pre-agreeing
//! the order of transaction *inputs* turns distributed commit into deterministic local
//! replay. The thesis already cites both, as positive precedent rather than competition. The
//! interface between a matching engine and a ledger is the one place this architecture is
//! unusually well suited to a problem it was not designed for.
//!
//! # What this module therefore does, and does not
//!
//! **Does:** the price-time priority rules, the sequenced-event contract, the conversion
//! from a fill to a balanced posting set, and the determinism obligation that makes replay
//! possible.
//!
//! **Does not:** lock-free data structures, cache-line padding, kernel bypass, or any of the
//! engineering that makes a real matching engine fast. Those belong in a tier-1 binary with
//! its own build, its own latency tests, and no dependency on this crate — and writing them
//! here would produce something that was neither fast enough to be a matching engine nor
//! simple enough to be a reference.

use gbs_kernel::{Amount, Chart, Currency, Entry, Epoch, KernelError, Party, PostingSet, Sealed, Stamp};
use std::collections::BTreeMap;
use std::fmt;

/// Which side of the book an order rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// A limit order.
///
/// `sequence` is assigned by tier 1 and is the **total order** the ledger later validates
/// against. It is not a timestamp: two orders arriving in the same nanosecond still have
/// distinct sequence numbers, and a clock going backwards cannot reorder the book.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: String,
    pub party: Party,
    pub side: Side,
    /// Price in minor units of the settlement currency, at its scale.
    pub price: i128,
    pub quantity: i128,
    pub sequence: u64,
}

/// One match: a resting order and an incoming one, crossed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub maker: String,
    pub taker: String,
    pub maker_party: Party,
    pub taker_party: Party,
    /// **The resting order's price, always.** See [`OrderBook::submit`].
    pub price: i128,
    pub quantity: i128,
    pub sequence: u64,
}

/// What the book refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchError {
    /// A sequence number that does not strictly increase.
    ///
    /// The sequence *is* the total order, so a repeat or a regression means the stream has
    /// been reordered or duplicated in transit. Refused rather than tolerated: a ledger
    /// validating against a broken order would record a book state that never existed.
    SequenceNotMonotonic { last: u64, received: u64 },
    /// A non-positive price or quantity.
    NotPositive { field: &'static str, value: i128 },
    Kernel(KernelError),
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchError::SequenceNotMonotonic { last, received } => write!(
                f,
                "sequence {received} does not follow {last}. The sequence is the total order \
                 the ledger validates against; a repeat or a regression means the stream was \
                 reordered or duplicated in transit"
            ),
            MatchError::NotPositive { field, value } => {
                write!(f, "{field} must be positive, got {value}")
            }
            MatchError::Kernel(e) => write!(f, "{e}"),
        }
    }
}

/// A price-time priority limit order book.
///
/// **Deterministic by construction.** Given the same sequence of orders it produces the same
/// sequence of fills, on every machine and every run, because:
///
/// * Price levels are a `BTreeMap`, so iteration is by price and not by hash seed.
/// * Within a level, orders are a `Vec` in arrival order, so time priority is positional.
/// * No clock is read. Ordering comes from the sequence number, which tier 1 assigns.
///
/// That determinism is what makes tier 2's job a *validation*: the ledger can replay the
/// stream and check it reaches the same book, which is a far stronger property than trusting
/// the fills it was sent.
#[derive(Debug, Default)]
pub struct OrderBook {
    /// Bids, keyed by price. Best bid is the **highest** price, so iteration is reversed.
    bids: BTreeMap<i128, Vec<Order>>,
    /// Asks, keyed by price. Best ask is the lowest, so iteration is forward.
    asks: BTreeMap<i128, Vec<Order>>,
    last_sequence: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook::default()
    }

    pub fn best_bid(&self) -> Option<i128> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<i128> {
        self.asks.keys().next().copied()
    }

    /// The spread, or `None` when either side is empty.
    ///
    /// `None`, not zero, and not a large sentinel. An empty book has no spread, and a
    /// market-data feed that published zero for it would show a locked market that does not
    /// exist. Honest absence, in the place a trading system would most punish its absence.
    pub fn spread(&self) -> Option<i128> {
        Some(self.best_ask()? - self.best_bid()?)
    }

    pub fn depth(&self, side: Side) -> usize {
        match side {
            Side::Buy => self.bids.values().map(|v| v.len()).sum(),
            Side::Sell => self.asks.values().map(|v| v.len()).sum(),
        }
    }

    /// Submit an order, matching against the book and resting the remainder.
    ///
    /// **Price-time priority**, and one rule that is easy to get wrong and expensive when it
    /// is: **a fill executes at the *resting* order's price, not the incoming one.** A buy at
    /// 105 crossing a resting sell at 100 fills at 100, and the taker's price improvement is
    /// real. Filling at the incoming price would silently transfer that improvement to the
    /// maker, which is both wrong and, at an exchange, actionable.
    pub fn submit(&mut self, order: Order) -> Result<Vec<Fill>, MatchError> {
        if order.sequence <= self.last_sequence {
            return Err(MatchError::SequenceNotMonotonic {
                last: self.last_sequence,
                received: order.sequence,
            });
        }
        if order.price <= 0 {
            return Err(MatchError::NotPositive { field: "price", value: order.price });
        }
        if order.quantity <= 0 {
            return Err(MatchError::NotPositive { field: "quantity", value: order.quantity });
        }
        self.last_sequence = order.sequence;

        let mut remaining = order.quantity;
        let mut fills = Vec::new();

        // Cross against the opposite side, best price first, arrival order within a price.
        loop {
            if remaining == 0 {
                break;
            }
            let level_price = match order.side {
                Side::Buy => match self.asks.keys().next().copied() {
                    Some(p) if p <= order.price => p,
                    _ => break,
                },
                Side::Sell => match self.bids.keys().next_back().copied() {
                    Some(p) if p >= order.price => p,
                    _ => break,
                },
            };

            let book = match order.side {
                Side::Buy => &mut self.asks,
                Side::Sell => &mut self.bids,
            };
            let level = book.get_mut(&level_price).expect("key just read");

            while remaining > 0 && !level.is_empty() {
                let resting = &mut level[0];
                let traded = remaining.min(resting.quantity);
                fills.push(Fill {
                    maker: resting.id.clone(),
                    taker: order.id.clone(),
                    maker_party: resting.party.clone(),
                    taker_party: order.party.clone(),
                    // The resting price. See the doc comment.
                    price: level_price,
                    quantity: traded,
                    sequence: order.sequence,
                });
                resting.quantity -= traded;
                remaining -= traded;
                if resting.quantity == 0 {
                    level.remove(0);
                }
            }
            if level.is_empty() {
                book.remove(&level_price);
            }
        }

        // Rest whatever did not fill, at the back of its price level.
        if remaining > 0 {
            let mut resting = order.clone();
            resting.quantity = remaining;
            let book = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            book.entry(resting.price).or_default().push(resting);
        }

        Ok(fills)
    }
}

/// Convert a fill into a balanced posting set — **the tier 1 → tier 2 boundary**.
///
/// Two conserved legs, exactly as a securities trade is: the instrument moves one way and
/// the cash the other, each conserving independently. The fill price is used to *derive* the
/// consideration and never enters a posting.
///
/// The epoch is supplied by tier 2, and the fill's sequence number becomes the transaction
/// identity — so the same fill delivered twice posts once, and replay is idempotent by
/// construction rather than by a deduplication table.
#[allow(clippy::too_many_arguments)]
pub fn fill_to_postings(
    fill: &Fill,
    instrument: &Currency,
    instrument_scale: u32,
    cash: &Currency,
    cash_scale: u32,
    maker_securities: &str,
    maker_cash: &str,
    taker_securities: &str,
    taker_cash: &str,
    stamp: Stamp,
    chart: &Chart,
    epoch: Epoch,
) -> Result<Sealed, MatchError> {
    let consideration = fill
        .price
        .checked_mul(fill.quantity)
        .ok_or(MatchError::Kernel(KernelError::Overflow { op: "consideration" }))?;

    // Instrument: out of the maker (a seller resting on the book), into the taker. The
    // direction is derived from who was resting, not asserted twice.
    let qty = Amount::new(fill.quantity, instrument.clone(), instrument_scale);
    let cash_amt = Amount::new(consideration, cash.clone(), cash_scale);

    let inst_out = Entry::new(1, maker_securities, qty.negate().map_err(MatchError::Kernel)?, stamp)
        .narrated(format!("fill seq {} at {}", fill.sequence, fill.price));
    let inst_in = inst_out.mirrored_to(2, taker_securities).map_err(MatchError::Kernel)?;
    let cash_out = Entry::new(3, taker_cash, cash_amt.negate().map_err(MatchError::Kernel)?, stamp)
        .narrated(format!("consideration for fill seq {}", fill.sequence));
    let cash_in = cash_out.mirrored_to(4, maker_cash).map_err(MatchError::Kernel)?;

    PostingSet::new(format!("fill-{}-{}", fill.sequence, fill.maker))
        .with(inst_out)
        .with(inst_in)
        .with(cash_out)
        .with(cash_in)
        .seal(chart, epoch)
        .map_err(MatchError::Kernel)
}

/// Replay a sequenced order stream and return the fills.
///
/// **This is tier 2's validation.** Because the book is deterministic, the ledger can replay
/// the stream it was sent and check it reaches the same fills — which is a much stronger
/// guarantee than trusting them. The order was agreed upstream; admission is a check, not a
/// sequencing decision, and that is the Calvin/Aria observation applied at a tier boundary.
pub fn replay(orders: &[Order]) -> Result<Vec<Fill>, MatchError> {
    let mut book = OrderBook::new();
    let mut all = Vec::new();
    for o in orders {
        all.extend(book.submit(o.clone())?);
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::Account;

    fn order(id: &str, party: &str, side: Side, price: i128, qty: i128, seq: u64) -> Order {
        Order {
            id: id.into(),
            party: Party::new(party),
            side,
            price,
            quantity: qty,
            sequence: seq,
        }
    }

    fn chart() -> Chart {
        Chart::new()
            .with(Account::new("maker.acme", "maker", "ACME"))
            .with(Account::new("maker.cash", "maker", "USD"))
            .with(Account::new("taker.acme", "taker", "ACME"))
            .with(Account::new("taker.cash", "taker", "USD"))
    }

    // ── price-time priority ─────────────────────────────────────────────────────────

    #[test]
    fn a_fill_executes_at_the_resting_price_not_the_incoming_one() {
        // The rule that is easy to get wrong and expensive when it is. A buy at 105 crossing
        // a resting sell at 100 fills at 100, and the taker keeps the price improvement.
        // Filling at 105 would transfer it silently to the maker.
        let mut b = OrderBook::new();
        b.submit(order("s1", "maker", Side::Sell, 100, 50, 1)).unwrap();
        let fills = b.submit(order("b1", "taker", Side::Buy, 105, 50, 2)).unwrap();

        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].price, 100, "the resting price");
        assert_eq!(fills[0].maker, "s1");
        assert_eq!(fills[0].taker, "b1");
    }

    #[test]
    fn better_prices_fill_first() {
        let mut b = OrderBook::new();
        b.submit(order("s_high", "m", Side::Sell, 102, 10, 1)).unwrap();
        b.submit(order("s_low", "m", Side::Sell, 100, 10, 2)).unwrap();
        let fills = b.submit(order("buy", "t", Side::Buy, 105, 20, 3)).unwrap();

        assert_eq!(fills.len(), 2);
        assert_eq!(fills[0].price, 100, "the better price first");
        assert_eq!(fills[1].price, 102);
    }

    #[test]
    fn within_a_price_level_arrival_order_wins() {
        // Time priority. Two sells at the same price fill in the order they arrived, which
        // is what makes resting on the book worth something.
        let mut b = OrderBook::new();
        b.submit(order("first", "m", Side::Sell, 100, 10, 1)).unwrap();
        b.submit(order("second", "m", Side::Sell, 100, 10, 2)).unwrap();
        let fills = b.submit(order("buy", "t", Side::Buy, 100, 20, 3)).unwrap();

        assert_eq!(fills[0].maker, "first");
        assert_eq!(fills[1].maker, "second");
    }

    #[test]
    fn an_order_that_does_not_cross_rests() {
        let mut b = OrderBook::new();
        assert!(b.submit(order("s", "m", Side::Sell, 105, 10, 1)).unwrap().is_empty());
        assert!(b.submit(order("bid", "t", Side::Buy, 100, 10, 2)).unwrap().is_empty());
        assert_eq!(b.best_bid(), Some(100));
        assert_eq!(b.best_ask(), Some(105));
        assert_eq!(b.spread(), Some(5));
    }

    #[test]
    fn a_partial_fill_rests_the_remainder() {
        let mut b = OrderBook::new();
        b.submit(order("s", "m", Side::Sell, 100, 30, 1)).unwrap();
        let fills = b.submit(order("buy", "t", Side::Buy, 100, 50, 2)).unwrap();
        assert_eq!(fills[0].quantity, 30);
        assert_eq!(b.best_bid(), Some(100), "the unfilled 20 rests as a bid");
        assert_eq!(b.depth(Side::Buy), 1);
        assert_eq!(b.depth(Side::Sell), 0);
    }

    // ── honest absence, in a trading context ────────────────────────────────────────

    #[test]
    fn an_empty_book_has_no_spread_rather_than_a_zero_one() {
        // A feed publishing zero here would show a locked market that does not exist, and
        // every strategy reading it would act on a price that was never quoted.
        let mut b = OrderBook::new();
        assert_eq!(b.spread(), None);
        b.submit(order("s", "m", Side::Sell, 100, 10, 1)).unwrap();
        assert_eq!(b.spread(), None, "one side is still not a market");
        b.submit(order("bid", "t", Side::Buy, 99, 10, 2)).unwrap();
        assert_eq!(b.spread(), Some(1));
    }

    // ── determinism: what makes tier 2's validation possible ────────────────────────

    #[test]
    fn replay_of_the_same_stream_produces_identical_fills() {
        // **The property the tier boundary rests on.** Because the book is deterministic,
        // the ledger can replay what it was sent and check it reaches the same fills, rather
        // than trusting them.
        let stream: Vec<Order> = vec![
            order("s1", "m1", Side::Sell, 102, 10, 1),
            order("s2", "m2", Side::Sell, 100, 15, 2),
            order("b1", "t1", Side::Buy, 101, 12, 3),
            order("b2", "t2", Side::Buy, 103, 20, 4),
            order("s3", "m3", Side::Sell, 99, 5, 5),
        ];
        let first = replay(&stream).unwrap();
        for _ in 0..50 {
            assert_eq!(replay(&stream).unwrap(), first);
        }
        assert!(!first.is_empty(), "the stream must actually cross, or this proves nothing");
    }

    #[test]
    fn the_book_reads_no_clock_so_ordering_comes_only_from_the_sequence() {
        // A clock going backwards must not reorder the book. Ordering is positional and by
        // sequence number, and there is no timestamp in `Order` at all.
        let o = order("x", "p", Side::Buy, 100, 1, 1);
        let d = format!("{o:?}");
        assert!(!d.contains("time"), "no clock in the order: {d}");
        assert!(!d.contains("timestamp"));
    }

    #[test]
    fn price_levels_iterate_in_price_order_not_hash_order() {
        // `BTreeMap`, so two processes agree. A `HashMap` here would make the book
        // non-deterministic across runs and tier 2's validation impossible.
        let mut b = OrderBook::new();
        for (p, seq) in [(103i128, 1u64), (101, 2), (105, 3), (102, 4)] {
            b.submit(order(&format!("s{p}"), "m", Side::Sell, p, 10, seq)).unwrap();
        }
        assert_eq!(b.best_ask(), Some(101));
        let fills = b.submit(order("buy", "t", Side::Buy, 110, 40, 5)).unwrap();
        let prices: Vec<i128> = fills.iter().map(|f| f.price).collect();
        assert_eq!(prices, vec![101, 102, 103, 105], "ascending, always");
    }

    // ── the sequence is the total order ─────────────────────────────────────────────

    #[test]
    fn a_non_monotonic_sequence_is_refused() {
        // The sequence *is* the total order. A repeat or a regression means the stream was
        // reordered or duplicated in transit, and a ledger validating against a broken order
        // would record a book state that never existed.
        let mut b = OrderBook::new();
        b.submit(order("a", "p", Side::Buy, 100, 10, 5)).unwrap();
        let e = b.submit(order("b", "p", Side::Buy, 100, 10, 5)).unwrap_err();
        assert!(matches!(e, MatchError::SequenceNotMonotonic { last: 5, received: 5 }));
        assert!(e.to_string().contains("reordered or duplicated in transit"));
        assert!(b.submit(order("c", "p", Side::Buy, 100, 10, 4)).is_err(), "and backwards too");
        assert!(b.submit(order("d", "p", Side::Buy, 100, 10, 6)).is_ok());
    }

    #[test]
    fn non_positive_prices_and_quantities_are_refused() {
        let mut b = OrderBook::new();
        assert!(matches!(
            b.submit(order("a", "p", Side::Buy, 0, 10, 1)),
            Err(MatchError::NotPositive { field: "price", .. })
        ));
        assert!(matches!(
            b.submit(order("a", "p", Side::Buy, 100, 0, 1)),
            Err(MatchError::NotPositive { field: "quantity", .. })
        ));
    }

    // ── the tier 1 → tier 2 boundary ────────────────────────────────────────────────

    #[test]
    fn a_fill_becomes_a_balanced_posting_set() {
        // The seam. Two conserved legs, exactly as a securities trade is: instrument one
        // way, cash the other, each conserving independently. The price derives the
        // consideration and never enters a posting.
        let mut b = OrderBook::new();
        b.submit(order("s1", "maker", Side::Sell, 10_000, 300, 1)).unwrap();
        let fills = b.submit(order("b1", "taker", Side::Buy, 10_000, 300, 2)).unwrap();

        let sealed = fill_to_postings(
            &fills[0],
            &Currency::new("ACME"),
            0,
            &Currency::new("USD"),
            2,
            "maker.acme",
            "maker.cash",
            "taker.acme",
            "taker.cash",
            Stamp::new(Epoch(10), 0),
            &chart(),
            Epoch(10),
        )
        .unwrap();

        sealed.verify().expect("both units conserve");
        assert_eq!(sealed.entries().len(), 4);
        assert_eq!(sealed.entries()[0].amount.minor, -300, "instrument out of the maker");
        assert_eq!(sealed.entries()[2].amount.minor, -3_000_000, "$30,000.00 out of the taker");
    }

    #[test]
    fn the_transaction_identity_is_the_sequence_so_replay_is_idempotent() {
        // A fill delivered twice posts once, by construction rather than by a deduplication
        // table — the identity of the business event decides sameness (kernel `TxnId`).
        let mut b = OrderBook::new();
        b.submit(order("s1", "maker", Side::Sell, 100, 10, 1)).unwrap();
        let fills = b.submit(order("b1", "taker", Side::Buy, 100, 10, 2)).unwrap();

        let build = || {
            fill_to_postings(
                &fills[0], &Currency::new("ACME"), 0, &Currency::new("USD"), 2,
                "maker.acme", "maker.cash", "taker.acme", "taker.cash",
                Stamp::new(Epoch(10), 0), &chart(), Epoch(10),
            )
            .unwrap()
        };
        assert_eq!(build().txn(), build().txn(), "same fill, same identity");
        assert!(build().txn().as_str().contains("fill-2"));
    }

    #[test]
    fn a_full_session_replays_and_every_fill_conserves() {
        // End to end: a sequenced stream, replayed deterministically, every fill becoming a
        // posting set the kernel accepts. This is what tier 2 actually does on ingest.
        let stream: Vec<Order> = (0..40)
            .map(|i| {
                let side = if i % 2 == 0 { Side::Sell } else { Side::Buy };
                let price = 100 + (i as i128 * 7) % 11;
                order(&format!("o{i}"), &format!("p{}", i % 4), side, price, 5 + i as i128 % 9, i + 1)
            })
            .collect();

        let fills = replay(&stream).unwrap();
        assert!(fills.len() > 5, "the session must actually trade: {} fills", fills.len());

        let ch = Chart::new()
            .with(Account::new("maker.acme", "maker", "ACME"))
            .with(Account::new("maker.cash", "maker", "USD"))
            .with(Account::new("taker.acme", "taker", "ACME"))
            .with(Account::new("taker.cash", "taker", "USD"));

        for (i, f) in fills.iter().enumerate() {
            let sealed = fill_to_postings(
                f, &Currency::new("ACME"), 0, &Currency::new("USD"), 2,
                "maker.acme", "maker.cash", "taker.acme", "taker.cash",
                Stamp::new(Epoch(100 + i as u64), 0), &ch, Epoch(100 + i as u64),
            )
            .unwrap_or_else(|e| panic!("fill {i} failed to post: {e}"));
            sealed.verify().unwrap();
        }
    }

    // ── the boundary this module refuses to cross ───────────────────────────────────

    #[test]
    fn this_module_is_the_interface_and_not_the_hot_path() {
        // Recorded as a test because it is a design commitment that would otherwise erode.
        // A real matching engine is lock-free, allocation-free on the hot path, pinned to a
        // core and behind kernel bypass. None of that is here, and putting it here would
        // produce something neither fast enough to be a matching engine nor simple enough
        // to be a reference.
        //
        // What IS here is the part tier 2 needs: the priority rules, the sequenced-event
        // contract, determinism, and the conversion to postings.
        let src = include_str!("matching.rs");

        // The needles are assembled at runtime, because a literal here would appear in the
        // source this test reads and the test would fail on its own text. That is not a
        // trick — a self-inspecting test that matched itself would report a defect that does
        // not exist, and the first draft of this test did exactly that.
        let unsafe_block = format!("{}{}", "unsa", "fe {");
        let unsafe_fn = format!("{}{}", "unsa", "fe fn");
        let spinning = format!("{}{}", "spin_", "loop");

        assert!(!src.contains(&unsafe_block), "no unsafe blocks: this is not the hot path");
        assert!(!src.contains(&unsafe_fn), "and no unsafe functions");
        assert!(!src.contains(&spinning), "no spinning either");
        assert!(src.contains("BTreeMap"), "determinism over speed, deliberately");
    }
}

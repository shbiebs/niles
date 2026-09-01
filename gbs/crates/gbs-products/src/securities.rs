//! Securities, funds, and ETFs.
//!
//! Equities, fixed income, multi-asset, and the fund structures over them. M4 does most of
//! the work, and this is the product line that tests it hardest — a fund's unit arithmetic
//! is the same conservation problem as a syndicated loan's, run daily, across thousands of
//! holders, for decades.
//!
//! # A holding is a balance in a different unit
//!
//! The insight that keeps this from needing an eighth mechanism: **a securities position is
//! a balance whose currency is an instrument.** 300 shares of a stock is 300 units of `ACME`
//! exactly as $300.00 is 300 units of USD at scale 2. Conservation applies unchanged — a
//! trade that creates or destroys shares is as wrong as one that creates or destroys dollars
//! — and the kernel already checks per-currency sums, so it checks per-instrument sums for
//! free.
//!
//! This is why `Currency` in the kernel is an opaque code with a scale rather than an ISO
//! enumeration. `ACME` at scale 0 is a whole-share instrument; `FUND-A` at scale 6 is a
//! mutual fund quoted in millionths of a unit. Neither needs the kernel to know what a
//! security is.
//!
//! # A trade is an FX transaction
//!
//! Buying 300 shares for $30,000 is two conserved legs sealed in one epoch: −$30,000 and
//! +300 ACME. It is structurally identical to selling dollars for euros, and it goes through
//! the same code, which is the second reason this product line needs nothing new. The rate
//! is a price, the legs conserve independently, and no rate ever enters a posting.
//!
//! # NAV, and the reason it is M7 rather than arithmetic
//!
//! A fund's net asset value per unit is (assets − liabilities) ÷ units outstanding — a
//! division, which is where money is created or destroyed. It must be reproducible at an
//! epoch, because a subscription priced at a NAV that cannot be recomputed is a subscription
//! nobody can audit. So NAV is a valuation with recorded inputs, not a number.

use gbs_kernel::{Amount, Chart, Currency, Entry, Epoch, KernelError, Party, PostingSet, Sealed, Stamp};
use gbs_mechanisms::{
    convert, ParticipantSet, ParticipationError, Priced, Rate, Rounding, ValuationError,
};

/// What a securities product refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecuritiesError {
    Kernel(KernelError),
    Valuation(ValuationError),
    Participation(ParticipationError),
    /// A trade whose cash and instrument legs are in the same unit.
    ///
    /// Refused: that is a transfer of the same thing to itself. It usually means the
    /// instrument code and the settlement currency were transposed.
    SameUnit { unit: Currency },
    /// A NAV computed against zero units outstanding.
    ///
    /// Refused rather than returning zero or infinity. A fund with no units has no NAV per
    /// unit, and the honest answer is that the question has no value — the same distinction
    /// `Reading::Absent` makes for a balance.
    NoUnitsOutstanding { fund: String },
    /// A redemption for more units than the holder owns.
    InsufficientUnits { holder: Party, held: Amount, requested: Amount },
}

impl std::fmt::Display for SecuritiesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritiesError::Kernel(e) => write!(f, "{e}"),
            SecuritiesError::Valuation(e) => write!(f, "{e}"),
            SecuritiesError::Participation(e) => write!(f, "{e}"),
            SecuritiesError::SameUnit { unit } => write!(
                f,
                "both legs of the trade are in {unit}; the instrument code and the \
                 settlement currency have probably been transposed"
            ),
            SecuritiesError::NoUnitsOutstanding { fund } => write!(
                f,
                "`{fund}` has no units outstanding, so it has no net asset value per unit. \
                 This is an absent value, not a zero one"
            ),
            SecuritiesError::InsufficientUnits { holder, held, requested } => write!(
                f,
                "`{holder}` holds {held} and cannot redeem {requested}"
            ),
        }
    }
}

/// Which way a trade goes, from the client's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// A settled trade: two conserved legs, one epoch.
#[derive(Debug, Clone)]
pub struct Trade {
    pub instrument: Currency,
    pub quantity: Amount,
    pub consideration: Amount,
    pub price: Rate,
    pub postings: Sealed,
}

/// Settle a securities trade.
///
/// Structurally an FX transaction: cash out, instrument in, each conserving on its own. The
/// price is a rate applied by [`convert`] to *derive* the consideration, and it never enters
/// a posting — a price inside an entry would balance against itself and no per-unit check
/// could see an error in it.
#[allow(clippy::too_many_arguments)]
pub fn settle_trade(
    txn: &str,
    side: Side,
    quantity: &Amount,
    price: Rate,
    cash_currency: &Currency,
    cash_scale: u32,
    client_cash: &str,
    client_securities: &str,
    street_cash: &str,
    street_securities: &str,
    stamp: Stamp,
    chart: &Chart,
    epoch: Epoch,
) -> Result<Trade, SecuritiesError> {
    if quantity.currency == *cash_currency {
        return Err(SecuritiesError::SameUnit { unit: cash_currency.clone() });
    }

    let priced: Priced = convert(quantity, cash_currency, price, cash_scale, Rounding::HalfEven)
        .map_err(SecuritiesError::Valuation)?;
    let consideration = priced.amount.clone();

    // A buy takes cash from the client and gives instrument; a sell is the mirror. Signs are
    // derived from `side` once, here, rather than being written out twice — the second
    // spelling is where a sign error lives.
    let sign = |a: &Amount, into_client: bool| -> Result<Amount, SecuritiesError> {
        let want_positive = match (side, into_client) {
            (Side::Buy, true) => true,   // instrument into the client
            (Side::Buy, false) => false, // cash out of the client
            (Side::Sell, true) => false, // instrument out of the client
            (Side::Sell, false) => true, // cash into the client
        };
        if want_positive {
            Ok(a.clone())
        } else {
            a.negate().map_err(SecuritiesError::Kernel)
        }
    };

    let inst_client = Entry::new(1, client_securities, sign(quantity, true)?, stamp)
        .narrated(format!("{side:?} {} at {price}", quantity.currency));
    let inst_street = inst_client.mirrored_to(2, street_securities).map_err(SecuritiesError::Kernel)?;
    let cash_client = Entry::new(3, client_cash, sign(&consideration, false)?, stamp)
        .narrated(format!("consideration for {}", quantity.currency));
    let cash_street = cash_client.mirrored_to(4, street_cash).map_err(SecuritiesError::Kernel)?;

    let postings = PostingSet::new(txn)
        .with(inst_client)
        .with(inst_street)
        .with(cash_client)
        .with(cash_street)
        .seal(chart, epoch)
        .map_err(SecuritiesError::Kernel)?;

    Ok(Trade {
        instrument: quantity.currency.clone(),
        quantity: quantity.clone(),
        consideration,
        price,
        postings,
    })
}

/// A fund or ETF: units outstanding, and the holders who own them.
pub struct Fund {
    pub id: String,
    /// The unit is an instrument like any other. `FUND-A` at scale 6 is a fund quoted in
    /// millionths of a unit.
    pub unit: Currency,
    pub unit_scale: u32,
    pub base_currency: Currency,
    pub base_scale: u32,
    /// The scale NAV is **quoted** at, which is finer than the currency's.
    ///
    /// Real funds price to four or more decimals against a two-decimal currency, and the
    /// reason is arithmetic rather than convention: a fund with a hundred million units and
    /// a NAV rounded to cents has a rounding error of up to half a cent per unit, which is
    /// half a million dollars of subscription mispricing. Quoting at the currency's scale
    /// is a real defect, and this field is what prevents it.
    ///
    /// Found by `nav_scales_the_numerator_before_dividing_so_precision_is_not_lost`, which
    /// asked a fund worth one cent across three units for its NAV and correctly got zero —
    /// correct at cent precision, and useless.
    pub nav_scale: u32,
    units_outstanding: Amount,
}

impl Fund {
    /// A fund quoting NAV at four decimals, which is the common convention.
    pub fn new(id: &str, unit: &str, unit_scale: u32, base: &str, base_scale: u32) -> Self {
        Fund::new_quoting_at(id, unit, unit_scale, base, base_scale, 4)
    }

    /// A fund quoting NAV at a stated scale. `nav_scale` MUST be at least `base_scale`;
    /// quoting a NAV more coarsely than the currency it is denominated in would round away
    /// value that exists.
    pub fn new_quoting_at(
        id: &str,
        unit: &str,
        unit_scale: u32,
        base: &str,
        base_scale: u32,
        nav_scale: u32,
    ) -> Self {
        assert!(
            nav_scale >= base_scale,
            "NAV cannot be quoted more coarsely than its own currency"
        );
        let u = Currency::new(unit);
        Fund {
            id: id.to_string(),
            unit: u.clone(),
            unit_scale,
            base_currency: Currency::new(base),
            base_scale,
            nav_scale,
            units_outstanding: Amount::new(0, u, unit_scale),
        }
    }

    pub fn units_outstanding(&self) -> &Amount {
        &self.units_outstanding
    }

    /// Net asset value **per unit**, as a valuation with recorded inputs.
    ///
    /// (assets − liabilities) ÷ units outstanding. A division, so the rounding is declared;
    /// and reproducible, so a subscription priced at this NAV can be recomputed years later.
    ///
    /// Refuses zero units outstanding rather than returning zero or dividing by zero. A fund
    /// with no units has no NAV per unit, and that is an absent value.
    pub fn nav_per_unit(
        &self,
        assets: &Amount,
        liabilities: &Amount,
        rounding: Rounding,
    ) -> Result<Rate, SecuritiesError> {
        if self.units_outstanding.minor == 0 {
            return Err(SecuritiesError::NoUnitsOutstanding { fund: self.id.clone() });
        }
        let net = assets
            .add(&liabilities.negate().map_err(SecuritiesError::Kernel)?)
            .map_err(SecuritiesError::Kernel)?;

        // Two liftings, and both are necessary.
        //
        // `unit_scale` converts the unit count from minor units to whole units, so the
        // division is per *unit* rather than per millionth of one. `nav_scale - base_scale`
        // lifts the answer from the currency's precision to the quoting precision, and it is
        // the one whose absence makes a large fund's NAV wrong by up to half a cent per unit.
        let numerator = net
            .minor
            .checked_mul(10i128.pow(self.unit_scale))
            .and_then(|n| n.checked_mul(10i128.pow(self.nav_scale - self.base_scale)))
            .ok_or(SecuritiesError::Kernel(KernelError::Overflow { op: "nav" }))?;
        let value = rounding
            .divide(numerator, self.units_outstanding.minor)
            .map_err(SecuritiesError::Kernel)?;
        Ok(Rate::new(value, self.nav_scale))
    }

    /// Create units against a subscription. The ETF creation-unit mechanism, and a mutual
    /// fund's subscription, are the same operation.
    ///
    /// Units are created, not transferred — so the posting set has a **unit issuance
    /// account** on the other side, and the fund's unit register conserves exactly as cash
    /// does. A fund that created units without a contra entry would be a fund whose unit
    /// count could not be reconciled against anything.
    pub fn subscribe(
        &mut self,
        holder_units_account: &str,
        units: &Amount,
        cash: &Amount,
        holder_cash: &str,
        fund_cash: &str,
        stamp: Stamp,
        chart: &Chart,
        epoch: Epoch,
    ) -> Result<Sealed, SecuritiesError> {
        let units_in = Entry::new(1, holder_units_account, units.clone(), stamp)
            .narrated(format!("subscription to {}", self.id));
        let units_issued = units_in
            .mirrored_to(2, format!("{}.units.issued", self.id))
            .map_err(SecuritiesError::Kernel)?;
        let cash_out = Entry::new(3, holder_cash, cash.negate().map_err(SecuritiesError::Kernel)?, stamp)
            .narrated("subscription consideration");
        let cash_in = cash_out.mirrored_to(4, fund_cash).map_err(SecuritiesError::Kernel)?;

        let sealed = PostingSet::new(format!("sub-{}-{}", self.id, epoch.0))
            .with(units_in)
            .with(units_issued)
            .with(cash_out)
            .with(cash_in)
            .seal(chart, epoch)
            .map_err(SecuritiesError::Kernel)?;

        self.units_outstanding =
            self.units_outstanding.add(units).map_err(SecuritiesError::Kernel)?;
        Ok(sealed)
    }

    /// Cancel units against a redemption. The exact mirror of [`Fund::subscribe`].
    #[allow(clippy::too_many_arguments)]
    pub fn redeem(
        &mut self,
        holder: &Party,
        holder_units_account: &str,
        held: &Amount,
        units: &Amount,
        cash: &Amount,
        holder_cash: &str,
        fund_cash: &str,
        stamp: Stamp,
        chart: &Chart,
        epoch: Epoch,
    ) -> Result<Sealed, SecuritiesError> {
        if units.minor > held.minor {
            return Err(SecuritiesError::InsufficientUnits {
                holder: holder.clone(),
                held: held.clone(),
                requested: units.clone(),
            });
        }
        let units_out = Entry::new(
            1,
            holder_units_account,
            units.negate().map_err(SecuritiesError::Kernel)?,
            stamp,
        )
        .narrated(format!("redemption from {}", self.id));
        let units_cancelled = units_out
            .mirrored_to(2, format!("{}.units.issued", self.id))
            .map_err(SecuritiesError::Kernel)?;
        let cash_in = Entry::new(3, holder_cash, cash.clone(), stamp).narrated("redemption proceeds");
        let cash_out = cash_in.mirrored_to(4, fund_cash).map_err(SecuritiesError::Kernel)?;

        let sealed = PostingSet::new(format!("red-{}-{}", self.id, epoch.0))
            .with(units_out)
            .with(units_cancelled)
            .with(cash_in)
            .with(cash_out)
            .seal(chart, epoch)
            .map_err(SecuritiesError::Kernel)?;

        self.units_outstanding = self
            .units_outstanding
            .add(&units.negate().map_err(SecuritiesError::Kernel)?)
            .map_err(SecuritiesError::Kernel)?;
        Ok(sealed)
    }
}

/// Distribute income across unit holders in proportion to their holdings.
///
/// A dividend, a coupon pass-through, or a fund distribution. M4 with the holders as the
/// participant set, which is exactly the syndicated-lending shape with different nouns — and
/// the reason both need the same exactness: a distribution that loses a cent per holder per
/// quarter loses a great deal over a fund's life, and it loses it from named people.
pub fn distribute(
    holders: &ParticipantSet,
    total: &Amount,
) -> Result<Vec<(Party, Amount)>, SecuritiesError> {
    holders.allocate(total).map_err(SecuritiesError::Participation)
}

/// Build a participant set from unit holdings.
///
/// The shares are `units_held / total_units` as exact rationals, so a holder with 1/3 of a
/// fund receives exactly a third and the odd cent goes to the declared residual holder.
pub fn holders_from_units(
    holdings: &[(Party, i128)],
    residual: &Party,
) -> Result<ParticipantSet, SecuritiesError> {
    let total: i128 = holdings.iter().map(|(_, u)| *u).sum();
    if total == 0 {
        return Err(SecuritiesError::Participation(ParticipationError::Empty));
    }
    let parties = holdings
        .iter()
        .map(|(p, u)| (p.clone(), gbs_mechanisms::Share::new(*u, total)))
        .collect();
    ParticipantSet::new(parties, residual.clone()).map_err(SecuritiesError::Participation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gbs_kernel::Account;

    fn usd(m: i128) -> Amount {
        Amount::minor_2dp(m, "USD")
    }
    fn shares(n: i128, code: &str) -> Amount {
        Amount::new(n, Currency::new(code), 0)
    }
    fn st(e: u64, d: i64) -> Stamp {
        Stamp::new(Epoch(e), d)
    }
    fn p(s: &str) -> Party {
        Party::new(s)
    }

    fn chart() -> Chart {
        Chart::new()
            .with(Account::new("client.cash", "client-1", "USD"))
            .with(Account::new("client.acme", "client-1", "ACME"))
            .with(Account::new("street.cash", "broker", "USD"))
            .with(Account::new("street.acme", "broker", "ACME"))
            .with(Account::new("holder.units", "holder-1", "FUND-A"))
            .with(Account::new("fund-a.units.issued", "fund-a", "FUND-A"))
            .with(Account::new("holder.cash", "holder-1", "USD"))
            .with(Account::new("fund.cash", "fund-a", "USD"))
    }

    // ── a holding is a balance in a different unit ──────────────────────────────────

    #[test]
    fn a_share_trade_conserves_in_both_units_independently() {
        // The claim this product line rests on: the kernel's per-currency check becomes a
        // per-instrument check with no change at all. 300 ACME and $30,000 each conserve on
        // their own, and neither can compensate for the other.
        let t = settle_trade(
            "trade-1",
            Side::Buy,
            &shares(300, "ACME"),
            Rate::new(100_00, 2), // $100.00 per share
            &Currency::new("USD"),
            2,
            "client.cash",
            "client.acme",
            "street.cash",
            "street.acme",
            st(10, 0),
            &chart(),
            Epoch(10),
        )
        .unwrap();

        t.postings.verify().expect("both units conserve");
        assert_eq!(t.consideration.minor, 3_000_000, "$30,000.00");
        assert_eq!(t.quantity.minor, 300);
        assert_eq!(t.postings.entries().len(), 4);
    }

    #[test]
    fn a_sell_is_the_exact_mirror_of_a_buy() {
        let buy = settle_trade(
            "b", Side::Buy, &shares(300, "ACME"), Rate::new(100_00, 2),
            &Currency::new("USD"), 2, "client.cash", "client.acme",
            "street.cash", "street.acme", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();
        let sell = settle_trade(
            "s", Side::Sell, &shares(300, "ACME"), Rate::new(100_00, 2),
            &Currency::new("USD"), 2, "client.cash", "client.acme",
            "street.cash", "street.acme", st(11, 0), &chart(), Epoch(11),
        )
        .unwrap();

        for (b, s) in buy.postings.entries().iter().zip(sell.postings.entries()) {
            assert_eq!(b.account, s.account);
            assert_eq!(b.amount.minor, -s.amount.minor, "every leg reverses");
        }
    }

    #[test]
    fn a_trade_whose_legs_are_in_one_unit_is_refused() {
        // Almost always a transposed instrument code and settlement currency.
        let e = settle_trade(
            "bad", Side::Buy, &usd(300), Rate::new(100_00, 2),
            &Currency::new("USD"), 2, "client.cash", "client.acme",
            "street.cash", "street.acme", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap_err();
        assert!(matches!(e, SecuritiesError::SameUnit { .. }));
        assert!(e.to_string().contains("transposed"));
    }

    #[test]
    fn the_price_never_enters_a_posting() {
        // The kernel rule (`ARCHITECTURE.md` §6) applied to securities: a price inside an
        // entry balances against itself and no per-unit check can see an error in it.
        let t = settle_trade(
            "t", Side::Buy, &shares(7, "ACME"), Rate::new(142_857, 3),
            &Currency::new("USD"), 2, "client.cash", "client.acme",
            "street.cash", "street.acme", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();
        for e in t.postings.entries() {
            // Every entry is in exactly one unit, and it is one of the two.
            assert!(
                e.amount.currency == Currency::new("ACME") || e.amount.currency == Currency::new("USD")
            );
        }
        t.postings.verify().unwrap();
    }

    #[test]
    fn a_fractional_price_rounds_once_and_still_conserves() {
        // $142.857 per share × 7 shares = $1,000.00 (rounded). The rounding happens once, in
        // the valuation, and both legs are built from the rounded figure — so there is no
        // path by which the cash leg and the client's debit differ.
        let t = settle_trade(
            "t", Side::Buy, &shares(7, "ACME"), Rate::new(142_857, 3),
            &Currency::new("USD"), 2, "client.cash", "client.acme",
            "street.cash", "street.acme", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();
        assert_eq!(t.consideration.minor, 100_000, "$1,000.00");
        t.postings.verify().unwrap();
    }

    // ── funds ───────────────────────────────────────────────────────────────────────

    #[test]
    fn subscribing_creates_units_that_conserve_against_an_issuance_account() {
        // A fund that created units with no contra entry would have a unit count nobody
        // could reconcile against anything.
        let mut f = Fund::new("fund-a", "FUND-A", 6, "USD", 2);
        let sealed = f
            .subscribe(
                "holder.units",
                &Amount::new(1_000_000, Currency::new("FUND-A"), 6), // 1.000000 units
                &usd(100_000),
                "holder.cash",
                "fund.cash",
                st(10, 0),
                &chart(),
                Epoch(10),
            )
            .unwrap();
        sealed.verify().unwrap();
        assert_eq!(sealed.entries().len(), 4, "units and cash, two legs each");
        assert_eq!(f.units_outstanding().minor, 1_000_000);
    }

    #[test]
    fn a_subscription_and_its_redemption_leave_the_fund_exactly_where_it_started() {
        let mut f = Fund::new("fund-a", "FUND-A", 6, "USD", 2);
        let units = Amount::new(1_234_567, Currency::new("FUND-A"), 6);
        f.subscribe("holder.units", &units, &usd(100_000), "holder.cash", "fund.cash", st(10, 0), &chart(), Epoch(10))
            .unwrap();
        f.redeem(
            &p("holder-1"), "holder.units", &units, &units, &usd(100_000),
            "holder.cash", "fund.cash", st(11, 0), &chart(), Epoch(11),
        )
        .unwrap();
        assert_eq!(f.units_outstanding().minor, 0);
    }

    #[test]
    fn redeeming_more_units_than_are_held_is_refused() {
        let mut f = Fund::new("fund-a", "FUND-A", 6, "USD", 2);
        let held = Amount::new(1_000_000, Currency::new("FUND-A"), 6);
        f.subscribe("holder.units", &held, &usd(100_000), "holder.cash", "fund.cash", st(10, 0), &chart(), Epoch(10))
            .unwrap();
        let too_many = Amount::new(1_000_001, Currency::new("FUND-A"), 6);
        assert!(matches!(
            f.redeem(&p("holder-1"), "holder.units", &held, &too_many, &usd(1), "holder.cash", "fund.cash", st(11, 0), &chart(), Epoch(11)),
            Err(SecuritiesError::InsufficientUnits { .. })
        ));
    }

    // ── NAV ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn nav_per_unit_is_a_division_with_declared_rounding() {
        let mut f = Fund::new("fund-a", "FUND-A", 6, "USD", 2);
        f.subscribe(
            "holder.units",
            &Amount::new(2_000_000, Currency::new("FUND-A"), 6), // 2 units
            &usd(200_000),
            "holder.cash", "fund.cash", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();

        // $2,500.00 of assets, $100.00 of liabilities, over 2 units = $1,200.0000 per unit,
        // quoted at four decimals.
        let nav = f
            .nav_per_unit(&usd(250_000), &usd(10_000), Rounding::HalfEven)
            .unwrap();
        assert_eq!(nav.scale, 4, "quoted finer than the currency");
        assert_eq!(nav.value, 1_200_0000);
        assert_eq!(nav.to_string(), "1200.0000");
    }

    #[test]
    fn a_fund_with_no_units_has_no_nav_rather_than_a_zero_one() {
        // Honest absence, in the place a division by zero would otherwise happen. A NAV of
        // zero would price a subscription at nothing.
        let f = Fund::new("empty", "FUND-E", 6, "USD", 2);
        let e = f.nav_per_unit(&usd(100_000), &usd(0), Rounding::HalfEven).unwrap_err();
        assert!(matches!(e, SecuritiesError::NoUnitsOutstanding { .. }));
        assert!(e.to_string().contains("absent value, not a zero one"));
    }

    #[test]
    fn nav_is_reproducible_across_repeated_computation() {
        // A subscription priced at a NAV that cannot be recomputed is one nobody can audit.
        let mut f = Fund::new("fund-a", "FUND-A", 6, "USD", 2);
        f.subscribe(
            "holder.units", &Amount::new(3_333_333, Currency::new("FUND-A"), 6),
            &usd(100_000), "holder.cash", "fund.cash", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();
        let first = f.nav_per_unit(&usd(987_654), &usd(12_345), Rounding::HalfEven).unwrap();
        for _ in 0..50 {
            assert_eq!(f.nav_per_unit(&usd(987_654), &usd(12_345), Rounding::HalfEven).unwrap(), first);
        }
    }

    #[test]
    fn nav_scales_the_numerator_before_dividing_so_precision_is_not_lost() {
        // One cent across three units. At the currency's own two decimals this is $0.00 per
        // unit — correct, and useless. Quoted at four it is $0.0033, which is the answer.
        //
        // This test found a real design gap: the first version quoted NAV at `base_scale`,
        // so a fund with a hundred million units would have had up to half a cent per unit
        // of rounding error in its subscription price. `nav_scale` exists because of it.
        let mut f = Fund::new("fund-a", "FUND-A", 6, "USD", 2);
        f.subscribe(
            "holder.units", &Amount::new(3_000_000, Currency::new("FUND-A"), 6),
            &usd(1), "holder.cash", "fund.cash", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();
        let nav = f.nav_per_unit(&usd(1), &usd(0), Rounding::Floor).unwrap();
        assert_eq!(nav.value, 33, "0.0033 per unit");
        assert_eq!(nav.scale, 4);

        // And a fund quoting at the currency's own scale gets the useless answer, which is
        // why `new` defaults to four.
        let mut coarse = Fund::new_quoting_at("fund-a", "FUND-A", 6, "USD", 2, 2);
        coarse.subscribe(
            "holder.units", &Amount::new(3_000_000, Currency::new("FUND-A"), 6),
            &usd(1), "holder.cash", "fund.cash", st(10, 0), &chart(), Epoch(10),
        )
        .unwrap();
        assert_eq!(coarse.nav_per_unit(&usd(1), &usd(0), Rounding::Floor).unwrap().value, 0);
    }

    // ── distributions ───────────────────────────────────────────────────────────────

    #[test]
    fn a_distribution_across_holders_conserves_exactly() {
        // The syndicated-lending property with different nouns. A dividend that loses a cent
        // per holder per quarter loses it from named people, for the life of the fund.
        let holdings = vec![(p("a"), 333_333i128), (p("b"), 333_333), (p("c"), 333_334)];
        let set = holders_from_units(&holdings, &p("a")).unwrap();

        for total in [1i128, 7, 101, 999_999, 123_456_789] {
            let d = distribute(&set, &usd(total)).unwrap();
            assert_eq!(d.iter().map(|(_, a)| a.minor).sum::<i128>(), total, "total={total}");
            assert_eq!(d.len(), 3);
        }
    }

    #[test]
    fn a_thousand_quarterly_distributions_lose_nothing() {
        // The cumulative form, which is the one that matters for a fund: a per-quarter
        // rounding drift is invisible in any single distribution and material over decades.
        let holdings: Vec<(Party, i128)> =
            (0..17).map(|i| (p(&format!("holder-{i}")), 1_000 + i as i128 * 37)).collect();
        let set = holders_from_units(&holdings, &p("holder-0")).unwrap();

        let mut distributed = 0i128;
        let mut declared = 0i128;
        for q in 0..1_000i128 {
            let amount = 10_000 + (q * 7919) % 99_991;
            declared += amount;
            distributed += distribute(&set, &usd(amount)).unwrap().iter().map(|(_, a)| a.minor).sum::<i128>();
        }
        assert_eq!(distributed, declared, "no drift across a thousand distributions");
    }

    #[test]
    fn holders_with_no_units_at_all_produce_an_error_rather_than_a_division_by_zero() {
        let e = holders_from_units(&[(p("a"), 0), (p("b"), 0)], &p("a")).unwrap_err();
        assert!(matches!(e, SecuritiesError::Participation(ParticipationError::Empty)));
    }

    #[test]
    fn a_single_holder_receives_the_whole_distribution() {
        let set = holders_from_units(&[(p("sole"), 500)], &p("sole")).unwrap();
        let d = distribute(&set, &usd(12_345)).unwrap();
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].1.minor, 12_345);
    }

    // ── the instrument is opaque to the kernel ──────────────────────────────────────

    #[test]
    fn instruments_at_different_scales_all_work_without_the_kernel_knowing_what_they_are() {
        // Whole shares at scale 0, a fund at scale 6, a bond quoted in 32nds — the kernel
        // sees three currencies with three scales and checks each independently. This is why
        // `Currency` is an opaque code rather than an ISO enumeration.
        for (code, scale, qty) in [("ACME", 0u32, 300i128), ("FUND-A", 6, 1_234_567), ("BOND-X", 5, 99_750)] {
            let a = Amount::new(qty, Currency::new(code), scale);
            let b = a.negate().unwrap();
            assert_eq!(a.add(&b).unwrap().minor, 0, "{code} conserves at scale {scale}");
        }
    }
}

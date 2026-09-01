//! FX and multi-currency: an atomic cross-currency transfer.
//!
//! The first product, chosen because it exercises the property the kernel exists to
//! provide. An FX transaction is **two conserved legs sealed in one epoch** — never an
//! amount multiplied by a rate inside a posting.
//!
//! The distinction is the whole of `ARCHITECTURE.md` §6's last bullet. A rate applied
//! inside an entry produces a line that balances against itself, so no per-currency check
//! can see an error in it. Two legs produce two independent obligations, and the kernel
//! checks both.

use gbs_kernel::{Amount, Chart, Currency, Entry, Epoch, KernelError, PostingSet, Sealed, Stamp};
use gbs_mechanisms::{convert, Rate, Rounding, ValuationError};

/// What an FX transfer can refuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FxError {
    Valuation(ValuationError),
    Kernel(KernelError),
    /// Both legs in the same currency. Refused: that is a transfer, and routing it through
    /// FX would apply a rate to a pair that needs none.
    SameCurrency { currency: Currency },
}

impl std::fmt::Display for FxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FxError::Valuation(e) => write!(f, "{e}"),
            FxError::Kernel(e) => write!(f, "{e}"),
            FxError::SameCurrency { currency } => {
                write!(f, "both legs are in {currency}; this is a transfer, not an FX trade")
            }
        }
    }
}

/// A completed FX trade: the two legs, and the rate that produced them.
#[derive(Debug, Clone)]
pub struct FxTrade {
    pub sold: Amount,
    pub bought: Amount,
    pub rate: Rate,
    pub postings: Sealed,
}

/// Execute an FX trade: sell `amount` from `sell_from`, buy the converted amount into
/// `buy_into`, with the bank's two nostro accounts on the other side.
///
/// Four entries, two currencies, one epoch. The kernel checks each currency independently,
/// so a rate error shows up as a residual in exactly one of them rather than balancing
/// invisibly.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    txn: &str,
    sell_from: &str,
    sell_nostro: &str,
    buy_into: &str,
    buy_nostro: &str,
    amount: &Amount,
    to: &Currency,
    to_scale: u32,
    rate: Rate,
    rounding: Rounding,
    stamp: Stamp,
    chart: &Chart,
    epoch: Epoch,
) -> Result<FxTrade, FxError> {
    if amount.currency == *to {
        return Err(FxError::SameCurrency { currency: to.clone() });
    }
    let priced = convert(amount, to, rate, to_scale, rounding).map_err(FxError::Valuation)?;

    // Leg one, in the sold currency: the customer's account down, the bank's nostro up.
    let sell_debit = Entry::new(1, sell_from, amount.negate().map_err(FxError::Kernel)?, stamp)
        .narrated(format!("fx sell {} at {}", amount.currency, rate));
    let sell_credit = sell_debit.mirrored_to(2, sell_nostro).map_err(FxError::Kernel)?;

    // Leg two, in the bought currency: the bank's nostro down, the customer's account up.
    let buy_debit = Entry::new(3, buy_nostro, priced.amount.negate().map_err(FxError::Kernel)?, stamp)
        .narrated(format!("fx buy {} at {}", priced.amount.currency, rate));
    let buy_credit = buy_debit.mirrored_to(4, buy_into).map_err(FxError::Kernel)?;

    let postings = PostingSet::new(txn)
        .with(sell_debit)
        .with(sell_credit)
        .with(buy_debit)
        .with(buy_credit)
        .seal(chart, epoch)
        .map_err(FxError::Kernel)?;

    Ok(FxTrade { sold: amount.clone(), bought: priced.amount, rate, postings })
}

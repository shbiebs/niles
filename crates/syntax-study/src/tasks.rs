//! **How each task is run on each surface** — the harness's half of the corpus. A program only
//! states the task; what it is called with, and what is read back, is fixed here, the same for
//! every surface.
//!
//! Conventions every program follows: a query or view is named `answer` (B03: `answer_valid`
//! and `answer_system`); a transaction is a function named `task`, taking the arguments below in
//! order; B03's correction is a function named `correct`. A SQL query task's program *is* the
//! query; a PRQL or DL program produces `answer`.

use crate::data::Dataset;
use crate::oracle::args::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Q,
    T,
    V,
    B,
}

pub fn class(task: &str) -> Class {
    match &task[..1] {
        "Q" => Class::Q,
        "T" => Class::T,
        "V" => Class::V,
        _ => Class::B,
    }
}

/// The surfaces, in the design's order, and the file extension of each one's programs.
pub const SURFACES: &[(&str, &str)] = &[
    ("SQL", "sql"),
    ("NL", "nl.niles"),
    ("RS", "rs.niles"),
    ("PRQL", "prql"),
    ("DL", "dl"),
];

/// Whether a surface is in scope for a task (design §2: PRQL and DL on the query tasks only).
pub fn in_scope(surface: &str, task: &str) -> bool {
    match surface {
        "PRQL" | "DL" => class(task) == Class::Q,
        _ => true,
    }
}

/// An argument of a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arg {
    Acct(i64),
    Money(i64, &'static str),
    Int(i64),
    /// A capability for an overdraft grant: the harness holds it and passes it in.
    Cap,
}

impl Arg {
    pub fn sql(&self) -> String {
        match self {
            Arg::Acct(a) | Arg::Int(a) => a.to_string(),
            Arg::Money(m, c) => format!("row({m})::{c}"),
            Arg::Cap => "1::auth_overdraft".into(),
        }
    }
}

/// The arguments of a transaction task, in order.
pub fn args(task: &str, d: &Dataset) -> Vec<Arg> {
    use Arg::*;
    match task {
        "T01" => vec![Acct(A), Acct(B), Money(AMOUNT, "usd")],
        "T02" => vec![
            Acct(A),
            Acct(B),
            Acct(FEE_ACCT),
            Money(AMOUNT, "usd"),
            Money(FEE, "usd"),
        ],
        "T03" => vec![Acct(A), Acct(B), Money(FX_USD, "usd"), Money(FX_EUR, "eur")],
        "T04" => vec![Acct(A), Acct(B), Money(HOLD, "usd"), Money(CAPTURE, "usd")],
        "T05" => vec![Acct(A), Money(HOLD, "usd")],
        "T06" => {
            let mut v = vec![Acct(BORROWER)];
            for (a, m) in LENDERS {
                v.push(Acct(a));
                v.push(Money(m, "usd"));
            }
            v
        }
        "T07" => vec![Acct(RECEIVABLE), Acct(INCOME), Money(ACCRUAL, "usd")],
        "T08" => {
            let owed: i64 = d
                .postings
                .iter()
                .filter(|p| p.acct == LOAN && p.cur == "usd")
                .map(|p| p.amt)
                .sum();
            vec![
                Acct(LOAN),
                Acct(PAYER),
                Acct(UNAPPLIED),
                Money(owed + EXCESS, "usd"),
            ]
        }
        "T09" => vec![Acct(A), Money(OVERDRAFT, "usd"), Cap],
        "T10" => vec![Int(REVERSED_TXN)],
        _ => Vec::new(),
    }
}

/// B03's correction arguments.
pub fn correction_args() -> Vec<Arg> {
    vec![
        Arg::Acct(CORR_FROM),
        Arg::Acct(CORR_TO),
        Arg::Money(CORR_AMT, "usd"),
    ]
}

/// The views a task's answer is read from, each with its tag (B03 only).
pub fn outputs(task: &str) -> Vec<(&'static str, Option<&'static str>)> {
    if task == "B03" {
        vec![
            ("answer_valid", Some("valid")),
            ("answer_system", Some("system")),
        ]
    } else {
        vec![("answer", None)]
    }
}

/// The SQL that reads a transaction's effect back: the legs appended after the dataset's
/// hundred transactions, or (T09) the grant.
pub fn sql_effect(task: &str) -> &'static str {
    if task == "T09" {
        "select acct, cur, (amt).minor from overdraft_limits"
    } else {
        "select acct, cur, coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor) \
         from postings where txn > 100"
    }
}

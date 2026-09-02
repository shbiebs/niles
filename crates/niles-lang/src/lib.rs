//! The stage-0 Niles compiler (thesis 7.2, Appendices B and E).
//!
//! Pipeline: lex -> parse -> resolve -> typecheck (currency rows, linearity,
//! effects, confidentiality) -> lower to IR.

pub mod ast;
pub mod currency_rows;
pub mod diagnostics;
pub mod effects;
pub mod keywords;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod postings;
pub mod resolve;
pub mod sexpr;
pub mod sql_surface;
pub mod typecheck;

//! The stage-0 Niles compiler (thesis 7.2, Appendices B and E).
//!
//! Pipeline: lex -> parse -> resolve -> typecheck (currency rows, linearity,
//! effects, confidentiality) -> lower to IR.

pub mod keywords;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod resolve;
pub mod typecheck;
pub mod effects;
pub mod currency_rows;
pub mod lower;
pub mod sql_surface;
pub mod diagnostics;

//! **Each surface's keyword list** (design §6: "shipped with the study as data, one list per
//! surface, from each language's documentation"). The lists are committed under
//! `corpus/keywords/`; `e30 keywords` regenerates them, each from the most authoritative
//! source this container holds for that language:
//!
//! * **SQL** — PostgreSQL 16's own list, `pg_get_keywords()` on the gate's server, plus
//!   PL/pgSQL's, which the server does not expose: transcribed from PostgreSQL 16's
//!   `src/pl/plpgsql/src/pl_reserved_kwlist.h` and `pl_unreserved_kwlist.h` (a transcription,
//!   and said so in the file's header).
//! * **Niles** (NL and RS) — the normative registry `niles_lang::keywords::KEYWORDS`, plus the
//!   pipeline's stage names, read from `StageKind::from_name` in `niles-lang/src/ast.rs`: a
//!   stage name is the pipeline's vocabulary exactly as a keyword is SQL's.
//! * **PRQL** — the lexer's keywords (PRQL 0.13 book, "Keywords"), plus every name the
//!   standard library declares, read out of the `std` module text embedded in the approved
//!   `prqlc` 0.13.14 binary itself.
//! * **DL** — Soufflé 2.5's directives, types, aggregates, functors and qualifiers,
//!   transcribed from the Soufflé documentation (a transcription, said so in the header).

use std::path::Path;

/// PL/pgSQL's reserved and unreserved keywords (PostgreSQL 16), transcribed.
const PLPGSQL: &[&str] = &[
    // pl_reserved_kwlist.h
    "all",
    "begin",
    "by",
    "case",
    "declare",
    "else",
    "end",
    "execute",
    "for",
    "foreach",
    "from",
    "if",
    "in",
    "into",
    "loop",
    "not",
    "null",
    "or",
    "strict",
    "then",
    "to",
    "using",
    "when",
    "while",
    // pl_unreserved_kwlist.h
    "absolute",
    "alias",
    "and",
    "array",
    "assert",
    "backward",
    "call",
    "chain",
    "close",
    "collate",
    "column",
    "column_name",
    "commit",
    "constant",
    "constraint",
    "constraint_name",
    "continue",
    "current",
    "cursor",
    "datatype",
    "debug",
    "default",
    "detail",
    "diagnostics",
    "do",
    "dump",
    "elseif",
    "elsif",
    "errcode",
    "error",
    "exception",
    "exit",
    "fetch",
    "first",
    "forward",
    "get",
    "hint",
    "import",
    "info",
    "insert",
    "is",
    "last",
    "log",
    "message",
    "message_text",
    "move",
    "next",
    "no",
    "notice",
    "open",
    "option",
    "perform",
    "pg_context",
    "pg_datatype_name",
    "pg_exception_context",
    "pg_exception_detail",
    "pg_exception_hint",
    "print_strict_params",
    "prior",
    "query",
    "raise",
    "relative",
    "return",
    "returned_sqlstate",
    "reverse",
    "rollback",
    "row_count",
    "rowtype",
    "schema",
    "schema_name",
    "scroll",
    "slice",
    "sqlstate",
    "stacked",
    "table",
    "table_name",
    "type",
    "use_column",
    "use_variable",
    "variable_conflict",
    "warning",
];

/// PRQL's lexer keywords and literal words (PRQL 0.13 book).
const PRQL_LEXER: &[&str] = &[
    "let", "into", "case", "prql", "type", "module", "internal", "func", "import", "enum", "true",
    "false", "null",
];

/// Soufflé 2.5, transcribed from its documentation: directives (written after a `.`), types,
/// aggregates, functors, bitwise and logical operators, qualifiers and representations.
const SOUFFLE: &[&str] = &[
    "decl",
    "input",
    "output",
    "printsize",
    "type",
    "comp",
    "init",
    "override",
    "functor",
    "pragma",
    "plan",
    "limitsize",
    "number",
    "symbol",
    "unsigned",
    "float",
    "count",
    "sum",
    "min",
    "max",
    "mean",
    "range",
    "cat",
    "contains",
    "match",
    "ord",
    "strlen",
    "substr",
    "to_number",
    "to_string",
    "to_float",
    "to_unsigned",
    "itou",
    "itof",
    "utoi",
    "utof",
    "ftoi",
    "ftou",
    "band",
    "bor",
    "bxor",
    "bshl",
    "bshr",
    "bshru",
    "bnot",
    "land",
    "lor",
    "lxor",
    "lnot",
    "nil",
    "true",
    "false",
    "as",
    "autoinc",
    "inline",
    "no_inline",
    "magic",
    "no_magic",
    "brie",
    "btree",
    "btree_delete",
    "eqrel",
    "choice-domain",
];

const AST: &str = include_str!("../../niles-lang/src/ast.rs");

/// The stage names `StageKind::from_name` recognises.
fn stage_names() -> Vec<String> {
    let start = AST.find("pub fn from_name").expect("from_name");
    let body = &AST[start..];
    let end = body.find("_ => Unknown").expect("the fallback arm");
    body[..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(String::from)
        .collect()
}

/// The names the `std` module declares, read from the text embedded in `prqlc`.
fn prql_std(prqlc: &Path) -> Result<Vec<String>, String> {
    let bytes = std::fs::read(prqlc).map_err(|e| format!("{}: {e}", prqlc.display()))?;
    let text: String = bytes
        .iter()
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' || *b == b'\n' {
                *b as char
            } else {
                '\u{0}'
            }
        })
        .collect();
    let start = text
        .find("let mul = left right -> internal std.mul")
        .ok_or("the std module text is not in this prqlc")?;
    let rest = &text[start..];
    let end = rest
        .find("module `prql`")
        .ok_or("the std module's end marker")?;
    let mut out = Vec::new();
    for line in rest[..end].lines() {
        let l = line.trim_start();
        for lead in ["let ", "module ", "type "] {
            if let Some(r) = l.strip_prefix(lead) {
                let name: String = r
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    out.push(name);
                }
            }
        }
    }
    Ok(out)
}

fn finish(header: &str, words: Vec<String>) -> String {
    let mut w: Vec<String> = words.into_iter().map(|s| s.to_lowercase()).collect();
    w.sort();
    w.dedup();
    format!("# {header}\n{}\n", w.join("\n"))
}

/// Generate the four files: `(file name, text)`.
pub fn generate(
    pg: &mut bank_bench::wire::Client,
    prqlc: &Path,
) -> Result<Vec<(&'static str, String)>, String> {
    let r = pg
        .simple("select word from pg_get_keywords()")
        .map_err(|e| e.to_string())?;
    let mut sql: Vec<String> = r.rows.into_iter().filter_map(|mut x| x.remove(0)).collect();
    let pg_n = sql.len();
    sql.extend(PLPGSQL.iter().map(|s| s.to_string()));
    let mut niles: Vec<String> = niles_lang::keywords::KEYWORDS
        .iter()
        .map(|k| k.word.to_string())
        .collect();
    let reg_n = niles.len();
    niles.extend(stage_names());
    let mut prql: Vec<String> = PRQL_LEXER.iter().map(|s| s.to_string()).collect();
    let std = prql_std(prqlc)?;
    let std_n = std.len();
    prql.extend(std);
    let dl: Vec<String> = SOUFFLE.iter().map(|s| s.to_string()).collect();
    Ok(vec![
        (
            "sql.txt",
            finish(
                &format!("SQL: pg_get_keywords() on PostgreSQL 16 ({pg_n} words), plus PL/pgSQL's keywords transcribed from pl_reserved_kwlist.h and pl_unreserved_kwlist.h"),
                sql,
            ),
        ),
        (
            "niles.txt",
            finish(
                &format!("Niles (NL and RS): niles_lang::keywords::KEYWORDS ({reg_n} words), plus the pipeline stage names of StageKind::from_name"),
                niles,
            ),
        ),
        (
            "prql.txt",
            finish(
                &format!("PRQL 0.13.14: the lexer's keywords, plus the {std_n} names the std module embedded in prqlc declares"),
                prql,
            ),
        ),
        (
            "dl.txt",
            finish(
                "Souffle 2.5: directives, types, aggregates, functors, operators and qualifiers, transcribed from its documentation",
                dl,
            ),
        ),
    ])
}

/// The keyword list a surface's constructs are counted against.
pub fn file_of(surface: &str) -> &'static str {
    match surface {
        "SQL" => "sql.txt",
        "PRQL" => "prql.txt",
        "DL" => "dl.txt",
        _ => "niles.txt",
    }
}

pub fn load(dir: &Path, surface: &str) -> Vec<String> {
    std::fs::read_to_string(dir.join(file_of(surface)))
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_stage_names_are_read_from_the_parser() {
        let s = super::stage_names();
        assert!(s.contains(&"group_by".to_string()) && s.contains(&"as_of".to_string()));
        assert!(!s.iter().any(|x| x.is_empty()));
    }
}

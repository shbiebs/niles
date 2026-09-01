fn main() {
    let schema = "schema bank {\n currency usd { scale: 2 }\n ledger postings {\n txn: TxnId, acct: Id<Account>, cur: Currency, amt: Money,\n idem: IdemKey window 30.days,\n conserve per (txn, cur);\n retain forever;\n }\n index ix_postings on postings (acct) anchor;\n}\n";
    let q = "select acct, sum(amt) from postings where acct = 7 group by acct";
    let program = format!("{schema}\nview __wire_result = sql {{ {q} }} serve {{ consistency: snapshot, materialize: auto }};\n");
    // warm
    for _ in 0..3 { let (p,_) = niles_lang::parser::parse_program(&program); let _ = niles_lang::resolve::resolve_program(&p, 1); }
    let n = 50;
    let t = std::time::Instant::now();
    for _ in 0..n {
        let (p, _) = niles_lang::parser::parse_program(&program);
        let (c, _) = niles_lang::resolve::resolve_program(&p, 1);
        let _ = niles_lang::typecheck::check_program(&p, &c);
    }
    let d = t.elapsed();
    println!("compile per query: {:.2} ms", d.as_secs_f64()*1000.0/n as f64);
    let t = std::time::Instant::now();
    for _ in 0..n { let _ = niles_lang::parser::parse_program(&program); }
    println!("  parse only:      {:.2} ms", t.elapsed().as_secs_f64()*1000.0/n as f64);
}

fn main() {
    let src = std::fs::read_to_string("bootstrap/lexer.niles").unwrap();
    let (prog, d) = niles_lang::parser::parse_program(&src);
    if d.has_errors() { eprintln!("{}", d.render(&src, "lexer.niles")); std::process::exit(1); }
    let mut it = niles_interp::Interp::new();
    it.load(&prog);
    match it.call("main", vec![]) {
        Ok(_) => { for l in &it.output { println!("{l}"); } }
        Err(e) => { eprintln!("ERR {:?} : {}", e.span(), e.message()); std::process::exit(2); }
    }
}

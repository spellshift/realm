fn main() {
    let code = "ssh('user:pass@host')";
    let tokens = eldritch_core::Lexer::new(code.to_string()).scan_tokens();
    let mut parser = eldritch_core::Parser::new(tokens);
    let stmts = parser.parse();
    println!("{:?}", stmts);
}

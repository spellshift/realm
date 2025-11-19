use eldritchv2::evaluator::Evaluator;
use eldritchv2::lexer::Lexer;
use eldritchv2::parser::Parser;
use std::io::{self, Write};

fn main() {
    let mut evaluator = Evaluator::new();
    loop {
        let mut input = String::new();
        loop {
            print!(">> ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break;
            }
            input.push_str(&line);
        }

        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if let Some(result) = evaluator.eval_program(&program) {
            println!("{:?}", result);
        }
    }
}

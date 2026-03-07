with open('implants/lib/eldritch/eldritch-wasm/src/browser.rs', 'r') as f:
    content = f.read()

import re

new_block = r"""        if is_complete {
            let payload = self.buffer.clone();

            // Check for meta function
            let parse_tokens = Lexer::new(payload.clone()).scan_tokens();
            let mut parser = eldritch_core::Parser::new(parse_tokens);
            let stmts = parser.parse();

            if stmts.0.len() == 1 {
                if let eldritch_core::StmtKind::Expression(expr) = &stmts.0[0].kind {
                    if let eldritch_core::ExprKind::Call(callee, args) = &expr.kind {
                        if let eldritch_core::ExprKind::Identifier(name) = &callee.kind {
                            if name == "ssh" && args.len() == 1 {
                                if let eldritch_core::Argument::Positional(arg_expr) = &args[0] {
                                    if let eldritch_core::ExprKind::Literal(eldritch_core::Value::String(val)) = &arg_expr.kind {
                                        // It's a single literal string argument
                                        self.buffer.clear();
                                        return format!(
                                            "{{ \"status\": \"meta\", \"function\": \"ssh\", \"args\": [{}] }}",
                                            format!("{:?}", val)
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            self.buffer.clear();
            return format!("{{ \"status\": \"complete\", \"payload\": {} }}", format!("{:?}", payload));
        }"""

content = re.sub(r'        if is_complete \{.*?\n        \}', new_block, content, flags=re.DOTALL)

with open('implants/lib/eldritch/eldritch-wasm/src/browser.rs', 'w') as f:
    f.write(content)

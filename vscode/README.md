# Eldritch Language Support for VS Code

Rich language support for **Eldritch** (`.eldritch` and `.tome`), the Pythonic/Starlark-inspired red team DSL for the [Realm](https://docs.realm.pub) Adversary Emulation Framework.

## Features

- **Syntax Highlighting**:
  - Full TextMate grammar for Eldritch `0.4.1` syntax (`def`, `if`/`elif`/`else`, `for`/`in`, `lambda`, type annotations, `f"..."` interpolated strings, `b"..."` byte strings, `r"..."` raw strings, triple-quoted strings, and comments).
  - Highlights all 26 Eldritch standard library modules (`aes`, `agent`, `assets`, `base64`, `chain`, `dns`, `file`, `gzip`, `hash`, `http`, `json`, `linux`, `net`, `portal`, `process`, `random`, `regex`, `registry`, `report`, `scan`, `ssh`, `sys`, `template`, `time`, `user`, `windows`), all 39 built-in functions, built-in type methods, `input_params`, and deprecated `0.4.0` functions.
- **Real-Time Linting & Diagnostics**:
  - Powered by the `eldritch-core` lexer, parser, and semantic analyzer (`eldritch-lsp`).
  - Detects syntax and indentation errors, undefined variables (with typo suggestions), unknown stdlib functions, deprecated `0.4.1` API calls (with migration guidance), removed `0.4.0` functions, call arity and keyword argument mismatches, invalid `return`/`break`/`continue` placement, unreachable code, duplicate dictionary keys, stdlib module shadowing, and undeclared `input_params` keys when a sibling `metadata.yml` is present.
- **Context-Aware Autocompletion**:
  - Completes standard library modules, module functions (`file.<Tab>`, `sys.<Tab>`, etc.) with parameter snippets and documentation, built-in functions, type methods (`str`, `Bytes`, `List`, `Dict`, `Set`), keywords, in-scope user functions/variables, and `input_params['...']` keys from sibling `metadata.yml` files.
- **Hover Documentation & Signature Help**:
  - Rich Markdown documentation and parameter signatures on hover and while typing function arguments.
- **Navigation**:
  - Go to Definition and Document Outline symbols for functions and variables.

## Quick Start

### Install from VSIX

```sh
code --install-extension vscode/eldritch.vsix
```

### Build from Source

```sh
# 1. Build the native Rust language server
cd implants && cargo build --release -p eldritch-lsp

# 2. Build and package the VS Code extension
cd ../vscode
npm install
npm run compile
npm run package
```


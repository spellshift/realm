

#[derive(Default)]
struct Stats {
    file: usize,
    error: usize,
    warning: usize,
    advice: usize,
    disabled: usize,
}

impl Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "{} files, {} errors, {} warnings, {} advices, {} disabled",
            self.file, self.error, self.warning, self.advice, self.disabled
        ))
    }
}


impl Stats {
    fn increment_file(&mut self) {
        self.file += 1;
    }

    fn increment(&mut self, x: EvalSeverity) {
        match x {
            EvalSeverity::Error => self.error += 1,
            EvalSeverity::Warning => self.warning += 1,
            EvalSeverity::Advice => self.advice += 1,
            EvalSeverity::Disabled => self.disabled += 1,
        }
    }
}

fn drain(xs: impl Iterator<Item = EvalMessage>, json: bool, stats: &mut Stats) {
    for x in xs {
        stats.increment(x.severity);
        if json {
            println!("{}", serde_json::to_string(&LintMessage::new(x)).unwrap());
        } else if let Some(error) = x.full_error_with_span {
            let mut error = error.to_owned();
            if !error.is_empty() && !error.ends_with('\n') {
                error.push('\n');
            }
            print!("{}", error);
        } else {
            println!("{}", x);
        }
    }
}

fn interactive(ctx: &Context) -> anyhow::Result<()> {
    let mut rl = ReadLine::new("STARLARK_RUST_HISTFILE");
    loop {
        match rl.read_line("$> ")? {
            Some(line) => {
                let mut stats = Stats::default();
                drain(ctx.expression(line).messages, false, &mut stats);
            }
            // User pressed EOF - disconnected terminal, or similar
            None => return Ok(()),
        }
    }
}

pub(crate) fn interactive_main() -> anyhow::Result<()> {
    let mut ctx = Context::new(
        if args.check {
            ContextMode::Check
        } else {
            ContextMode::Run
        },
        !args.evaluate.is_empty() || is_interactive,
        &expand_dirs(ext, args.prelude).collect::<Vec<_>>(),
        is_interactive,
    )?;

    Ok(interactive(&ctx)?)
}


impl Context {
    pub(crate) fn new(
        mode: ContextMode,
        print_non_none: bool,
        prelude: &[PathBuf],
        module: bool,
    ) -> anyhow::Result<Self> {
        let globals = globals();
        let prelude = prelude.try_map(|x| {
            let env = Module::new();

            let mut eval = Evaluator::new(&env);
            let module = AstModule::parse_file(x, &dialect())?;
            eval.eval_module(module, &globals)?;
            env.freeze()
        })?;

        let module = if module {
            Some(Self::new_module(&prelude))
        } else {
            None
        };
        let mut builtins: HashMap<LspUrl, Vec<Doc>> = HashMap::new();
        let mut builtin_symbols: HashMap<String, LspUrl> = HashMap::new();
        for doc in get_registered_docs() {
            let uri = Self::url_for_doc(&doc);
            builtin_symbols.insert(doc.id.name.clone(), uri.clone());
            builtins.entry(uri).or_default().push(doc);
        }
        let builtin_docs = builtins
            .into_iter()
            .map(|(u, ds)| (u, render_docs_as_code(&ds).join("\n\n")))
            .collect();

        Ok(Self {
            mode,
            print_non_none,
            prelude,
            module,
            builtin_docs,
            builtin_symbols,
        })
    }

    fn url_for_doc(doc: &Doc) -> LspUrl {
        let url = match &doc.item {
            DocItem::Module(_) => Url::parse("starlark:/native/builtins.bzl").unwrap(),
            DocItem::Object(_) => {
                Url::parse(&format!("starlark:/native/builtins/{}.bzl", doc.id.name)).unwrap()
            }
            DocItem::Function(_) => Url::parse("starlark:/native/builtins.bzl").unwrap(),
        };
        LspUrl::try_from(url).unwrap()
    }

    fn new_module(prelude: &[FrozenModule]) -> Module {
        let module = Module::new();
        for p in prelude {
            module.import_public_symbols(p);
        }
        module
    }

    fn go(&self, file: &str, ast: AstModule) -> EvalResult<impl Iterator<Item = EvalMessage>> {
        let mut warnings = Either::Left(iter::empty());
        let mut errors = Either::Left(iter::empty());
        let final_ast = match self.mode {
            ContextMode::Check => {
                warnings = Either::Right(self.check(&ast));
                Some(ast)
            }
            ContextMode::Run => {
                errors = Either::Right(self.run(file, ast).messages);
                None
            }
        };
        EvalResult {
            messages: warnings.chain(errors),
            ast: final_ast,
        }
    }

    // Convert an anyhow over iterator of EvalMessage, into an iterator of EvalMessage
    fn err(
        file: &str,
        result: anyhow::Result<EvalResult<impl Iterator<Item = EvalMessage>>>,
    ) -> EvalResult<impl Iterator<Item = EvalMessage>> {
        match result {
            Err(e) => EvalResult {
                messages: Either::Left(iter::once(EvalMessage::from_anyhow(Path::new(file), &e))),
                ast: None,
            },
            Ok(res) => EvalResult {
                messages: Either::Right(res.messages),
                ast: res.ast,
            },
        }
    }

    pub(crate) fn expression(
        &self,
        content: String,
    ) -> EvalResult<impl Iterator<Item = EvalMessage>> {
        let file = "expression";
        Self::err(
            file,
            AstModule::parse(file, content, &dialect()).map(|module| self.go(file, module)),
        )
    }

    pub(crate) fn file(&self, file: &Path) -> EvalResult<impl Iterator<Item = EvalMessage>> {
        let filename = &file.to_string_lossy();
        Self::err(
            filename,
            fs::read_to_string(file)
                .map(|content| self.file_with_contents(filename, content))
                .map_err(|e| e.into()),
        )
    }

    pub(crate) fn file_with_contents(
        &self,
        filename: &str,
        content: String,
    ) -> EvalResult<impl Iterator<Item = EvalMessage>> {
        Self::err(
            filename,
            AstModule::parse(filename, content, &dialect()).map(|module| self.go(filename, module)),
        )
    }

    fn run(&self, file: &str, ast: AstModule) -> EvalResult<impl Iterator<Item = EvalMessage>> {
        let new_module;
        let module = match self.module.as_ref() {
            Some(module) => module,
            None => {
                new_module = Self::new_module(&self.prelude);
                &new_module
            }
        };
        let mut eval = Evaluator::new(module);
        eval.enable_terminal_breakpoint_console();
        let globals = globals();
        Self::err(
            file,
            eval.eval_module(ast, &globals).map(|v| {
                if self.print_non_none && !v.is_none() {
                    println!("{}", v);
                }
                EvalResult {
                    messages: iter::empty(),
                    ast: None,
                }
            }),
        )
    }

    fn check(&self, module: &AstModule) -> impl Iterator<Item = EvalMessage> {
        let mut globals = Vec::new();
        for x in &self.prelude {
            globals.extend(x.names().map(|s| s.as_str()));
        }
        let globals = if self.prelude.is_empty() {
            None
        } else {
            Some(globals.as_slice())
        };

        module.lint(globals).into_iter().map(EvalMessage::from)
    }
}

impl LspContext for Context {
    fn parse_file_with_contents(&self, uri: &LspUrl, content: String) -> LspEvalResult {
        match uri {
            LspUrl::File(uri) => {
                let EvalResult { messages, ast } =
                    self.file_with_contents(&uri.to_string_lossy(), content);
                LspEvalResult {
                    diagnostics: messages.map(Diagnostic::from).collect(),
                    ast,
                }
            }
            _ => LspEvalResult::default(),
        }
    }

    fn resolve_load(&self, path: &str, current_file: &LspUrl) -> anyhow::Result<LspUrl> {
        let path = PathBuf::from(path);
        match current_file {
            LspUrl::File(current_file_path) => {
                let current_file_dir = current_file_path.parent();
                let absolute_path = match (current_file_dir, path.is_absolute()) {
                    (_, true) => Ok(path),
                    (Some(current_file_dir), false) => Ok(current_file_dir.join(&path)),
                    (None, false) => Err(ResolveLoadError::MissingCurrentFilePath(path)),
                }?;
                Ok(Url::from_file_path(absolute_path).unwrap().try_into()?)
            }
            _ => Err(
                ResolveLoadError::WrongScheme("file://".to_owned(), current_file.clone()).into(),
            ),
        }
    }

    fn resolve_string_literal(
        &self,
        literal: &str,
        current_file: &LspUrl,
    ) -> anyhow::Result<Option<StringLiteralResult>> {
        self.resolve_load(literal, current_file).map(|url| {
            Some(StringLiteralResult {
                url,
                location_finder: None,
            })
        })
    }

    fn get_load_contents(&self, uri: &LspUrl) -> anyhow::Result<Option<String>> {
        match uri {
            LspUrl::File(path) => match path.is_absolute() {
                true => match fs::read_to_string(&path) {
                    Ok(contents) => Ok(Some(contents)),
                    Err(e)
                        if e.kind() == io::ErrorKind::NotFound
                            || e.kind() == io::ErrorKind::NotADirectory =>
                    {
                        Ok(None)
                    }
                    Err(e) => Err(e.into()),
                },
                false => Err(LoadContentsError::NotAbsolute(uri.clone()).into()),
            },
            LspUrl::Starlark(_) => Ok(self.builtin_docs.get(uri).cloned()),
            _ => Err(LoadContentsError::WrongScheme("file://".to_owned(), uri.clone()).into()),
        }
    }

    fn get_url_for_global_symbol(
        &self,
        _current_file: &LspUrl,
        symbol: &str,
    ) -> anyhow::Result<Option<LspUrl>> {
        Ok(self.builtin_symbols.get(symbol).cloned())
    }
}

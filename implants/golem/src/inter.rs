/*
 * Copyright 2019 The Starlark in Rust Authors.
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//
// Plugins
#![cfg_attr(feature = "gazebo_lint", feature(plugin))]
#![cfg_attr(feature = "gazebo_lint", allow(deprecated))] // :(
#![cfg_attr(feature = "gazebo_lint", plugin(gazebo_lint))]
// Disagree these are good hints
#![allow(clippy::type_complexity)]

use std::fmt;
use std::fmt::Display;

use starlark::errors::EvalMessage;
use starlark::errors::EvalSeverity;
use starlark::read_line::ReadLine;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use anyhow;
use serde_json;

use eval::Context;
use eval::ContextMode;
use types::LintMessage;

mod eval;
mod types;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "starlark",
    about = "Evaluate Starlark code",
    global_settings(&[AppSettings::ColoredHelp]),
)]
struct Args {

}

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
            eprint!("{}", error);
        } else {
            println!("{}", x);
        }
    }
}

fn interactive(ctx: &Context) -> anyhow::Result<()> {
    let mut rl = ReadLine::new("STARLARK_RUST_HISTFILE")?;
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

pub fn interactive_main() -> anyhow::Result<()> {
    let ctx = Context::new(
        ContextMode::Run,
        true,
        &vec![],
        true,
    )?;

    interactive(&ctx)?;
    Ok(())
}
// ⚠️ 此文件被 build.rs 通过 include! 引入，禁止添加 use crate:: 依赖。
// 如需 crate 级别引用，请在 cli.rs 中添加。

use std::path::PathBuf;

use clap::Parser;

/// View opencode session exports in a pager, like `git log`.
#[derive(Parser)]
#[command(name = "ocat", about, version)]
pub struct Cli {
    /// Path to a session JSON file (reads from stdin if omitted)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Pager to use (default: $PAGER or `less -FRX`)
    #[arg(short = 'P', long, value_name = "PAGER")]
    pub pager: Option<String>,

    /// Disable pager, print directly to stdout
    #[arg(long)]
    pub no_pager: bool,

    /// Increase log verbosity (-v debug, -vv trace)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode, only output errors
    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::SetTrue, conflicts_with = "verbose")]
    pub quiet: bool,
}

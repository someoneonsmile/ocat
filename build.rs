// 构建脚本：生成 shell 补全文件和 man 手册页
// 通过 SHELL_HELP_DIR 环境变量控制输出目录
// 如果未设置，静默跳过（普通 cargo build 不生成这些文件）

include!("src/cli_types.rs");

use std::env;
use std::error::Error;
use std::fs::{File, create_dir_all};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate_to};
use clap_mangen::Man;

fn main() -> Result<(), Box<dyn Error>> {
    let outdir = match env::var_os("SHELL_HELP_DIR") {
        None => return Ok(()),
        Some(outdir) => std::path::PathBuf::from(outdir),
    };

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_owned();

    // 为所有支持的 shell 生成补全文件
    let complete_dir = outdir.join("complete");
    create_dir_all(&complete_dir)?;
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, &bin_name, &complete_dir)?;
    }

    // 生成 man 手册页
    let man_dir = outdir.join("man");
    create_dir_all(&man_dir)?;
    let mut man_out = File::create(man_dir.join(format!("{bin_name}.1")))?;
    let man = Man::new(cmd);
    man.render(&mut man_out)?;

    Ok(())
}

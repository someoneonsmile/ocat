use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::{self, Command, Stdio};
use std::sync::mpsc;
use std::thread;

use anyhow::Context;
use clap::Parser;
use env_logger::Env;
use log::{debug, error};

use crate::cli::Cli;

mod cli;
mod cli_types;
mod render;
mod types;

fn main() {
    increase_pipe_size();

    let (tx, rx) = mpsc::channel::<io::Result<String>>();
    thread::spawn(move || {
        let mut buf = String::new();
        let result = io::stdin().lock().read_to_string(&mut buf).map(|_| buf);
        tx.send(result).ok();
    });

    let cli = Cli::parse();

    let default_log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "warn",
            1 => "debug",
            _ => "trace",
        }
    };
    env_logger::Builder::from_env(Env::default().default_filter_or(default_log_level))
        .default_format()
        .format_level(true)
        .format_target(false)
        .format_module_path(false)
        .format_timestamp(None)
        .init();

    let input = match read_input(&cli, &rx).context("读取输入失败") {
        Ok(data) => data,
        Err(e) => {
            error!("{:#}", e);
            process::exit(1);
        }
    };

    debug!("读取到 {} 字节的输入", input.len());

    let output = match render::render_session(&input).context("渲染失败") {
        Ok(text) => text,
        Err(e) => {
            error!("{:#}", e);
            process::exit(1);
        }
    };

    if cli.no_pager {
        print!("{output}");
        return;
    }

    let pager = resolve_pager(&cli);
    run_pager(&pager, &output).unwrap_or_else(|e| {
        debug!("pager 启动失败: {:#}，降级为 stdout 输出", e);
        print!("{output}");
    });
}

/// 扩大 stdin 管道缓冲区（默认 64 KiB → 1 MiB）。
///
/// 这是针对上游 opencode export 已知 bug 的临时方案：
///   opencode export 使用 process.stdout.write() 输出 JSON 后，在 src/index.ts
///   的 finally 块中无条件调用 process.exit()，不等待 stdout 的 drain 事件。
///   当 stdout 是管道且数据量超过 stream 的 highWaterMark（64 KiB）时，
///   write() 返回 false 后的排队数据在 process.exit() 时被丢弃，导致 JSON 截断。
///
/// 将管道缓冲区扩大到 1 MiB 可让 opencode 在触发 backpressure 之前写完大部分数据，
///   从而绕开该 bug 路径。相关 issue：
///   - https://github.com/anomalyco/opencode/issues/29330
///   - https://github.com/anomalyco/opencode/issues/26399
///   - https://github.com/anomalyco/opencode/pull/39577 （修复 PR，未合并）
///
/// TODO: 上游 issue 解决后删除此函数及其调用。
fn increase_pipe_size() {
    let fd = io::stdin().as_raw_fd();
    unsafe {
        libc::fcntl(fd, libc::F_SETPIPE_SZ, 1_048_576);
    }
}

/// 从文件或后台 stdin 线程读取输入
fn read_input(cli: &Cli, rx: &mpsc::Receiver<io::Result<String>>) -> anyhow::Result<String> {
    match &cli.file {
        Some(path) => Ok(std::fs::read_to_string(path)?),
        None => Ok(rx.recv()??),
    }
}

/// 解析要使用的 pager
fn resolve_pager(cli: &Cli) -> String {
    if let Some(ref pager) = cli.pager {
        return pager.clone();
    }
    if let Ok(pager) = std::env::var("PAGER")
        && !pager.is_empty()
    {
        return ensure_less_frx(&pager);
    }
    "less -FRX".to_string()
}

fn ensure_less_frx(pager_cmd: &str) -> String {
    let parts: Vec<&str> = pager_cmd.split_whitespace().collect();
    if parts.is_empty() {
        return pager_cmd.to_string();
    }

    let cmd_name = parts[0].rsplit('/').next().unwrap_or(parts[0]);
    if cmd_name != "less" {
        return pager_cmd.to_string();
    }

    let existing_flags: std::collections::HashSet<char> = parts[1..]
        .iter()
        .filter(|s| s.starts_with('-') && !s.starts_with("--"))
        .flat_map(|s| s[1..].chars())
        .collect();

    let mut extra = String::new();
    for &c in &['F', 'R', 'X'] {
        if !existing_flags.contains(&c) {
            extra.push(c);
        }
    }

    if extra.is_empty() {
        pager_cmd.to_string()
    } else {
        format!("{pager_cmd} -{extra}")
    }
}

/// 通过 pager 输出内容
fn run_pager(pager_cmd: &str, content: &str) -> anyhow::Result<()> {
    let mut parts = pager_cmd.split_whitespace();
    let program = parts.next().context("pager 命令为空")?;
    let args: Vec<&str> = parts.collect();

    let mut child = Command::new(program)
        .args(&args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("无法启动 pager: {pager_cmd}"))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(content.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("pager 异常退出: {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_pager_default() {
        let cli = Cli {
            file: None,
            pager: None,
            no_pager: false,
            verbose: 0,
            quiet: false,
        };
        unsafe {
            std::env::remove_var("PAGER");
        }
        assert_eq!(resolve_pager(&cli), "less -FRX");
    }

    #[test]
    fn test_resolve_pager_custom() {
        let cli = Cli {
            file: None,
            pager: Some("bat".to_string()),
            no_pager: false,
            verbose: 0,
            quiet: false,
        };
        assert_eq!(resolve_pager(&cli), "bat");
    }

    #[test]
    fn test_ensure_less_frx_plain_less() {
        assert_eq!(ensure_less_frx("less"), "less -FRX");
    }

    #[test]
    fn test_ensure_less_frx_less_with_r() {
        assert_eq!(ensure_less_frx("less -R"), "less -R -FX");
    }

    #[test]
    fn test_ensure_less_frx_non_less() {
        assert_eq!(ensure_less_frx("bat"), "bat");
    }

    #[test]
    fn test_ensure_less_frx_full_path() {
        assert_eq!(ensure_less_frx("/usr/bin/less"), "/usr/bin/less -FRX");
    }

    #[test]
    fn test_ensure_less_frx_already_full() {
        assert_eq!(ensure_less_frx("less -FRX"), "less -FRX");
    }
}

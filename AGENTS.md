# AGENTS.md — ocat

View opencode session exports in a pager, like `git log`. Single-binary Rust CLI.

## Commands (use `just`)

```bash
just check        # cargo check (fast, no binary)
just build        # cargo build (debug)
just release      # cargo build --release
just test         # cargo test
just fmt          # cargo fmt
just fmt-check    # cargo fmt --check
just lint         # cargo clippy
just lint-strict  # cargo clippy -- -D warnings
just fix          # cargo clippy --fix --allow-dirty
just ci           # full gate: fmt-check → lint-strict → test → release build
just install      # cargo build --release → /usr/local/bin/ocat
```

Run with args: `just run -- --help` or `just run-release -- --help`.

## Toolchain

- **Rust edition 2024** → requires Rust **≥1.85**
- CI uses `dtolnay/rust-toolchain@stable` with `rustfmt` + `clippy` components
- No custom `rustfmt.toml` or `clippy.toml` — all tool defaults

## CI / Release

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | push/PR to `main` | `fmt-check`, `lint-strict`, `cargo test` |
| `release.yml` | tag `v*` | calls `build.yml` (all targets), publishes GitHub Release, auto-updates AUR |
| `nightly.yml` | daily cron | same as release but tagged `nightly` (prerelease, rolling), auto-updates nightly AUR |

### Build matrix (release)

| Target | OS | Note |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | native |
| `x86_64-unknown-linux-musl` | ubuntu-latest | uses `cross` tool, not cargo |
| `aarch64-unknown-linux-gnu` | ubuntu-24.04-arm | native |
| `x86_64-apple-darwin` | macos-latest | native |
| `aarch64-apple-darwin` | macos-latest | native |
| `x86_64-pc-windows-msvc` | windows-latest | native |

Artifact naming: `ocat-{target}.exe` on Windows, `ocat-{target}` elsewhere.

## Architecture

Single file `src/main.rs` + module split:

- **`src/cli_types.rs`**: clap derive — CLI 参数定义，被 build.rs 通过 include! 复用
- **`src/cli.rs`**: pub use re-export
- **`src/main.rs`**: 所有业务逻辑
  - **`main()`**: parse args → read input → render → output to pager
  - **`read_input()`**: 从文件或 stdin 读取 JSON
  - **`render()`**: 核心渲染逻辑（JSON → 人类可读文本）
  - **`resolve_pager()`**: 确定使用哪个 pager
  - **`run_pager()`**: fork+exec pager 进程并管道输出

## AUR

项目维护两个 AUR 包：

| 包名 | 子目录 | 远程 remote | 发布触发 |
|---|---|---|---|
| `ocat-bin` (稳定版) | `aur/` | `aur` → `ocat-bin.git` | `release.yml` (tag `v*`) |
| `ocat-nightly-bin` (每夜构建) | `aur-nightly/` | `aur-nightly` → `ocat-nightly-bin.git` | `nightly.yml` (每日 cron) |

两者通过 `git subtree` 维护，共享同一个 AUR 账户的 SSH key。

```bash
# 稳定版
just aur-srcinfo                # 重新生成 aur/.SRCINFO
just aur-release VERSION        # 更新版本号 + 推送

# 每夜构建
just aur-nightly-srcinfo        # 重新生成 aur-nightly/.SRCINFO
just aur-nightly-release DATE   # 更新日期版本号（YYYYMMDD）+ 推送
```

## Conventions

- Commit messages: `type: 中文描述` (e.g., `feat: 添加 JSON 渲染逻辑`, `fix: 修复 stdin 读取空输入崩溃`)
- PR branch → `main`; only release tags (`v*`) trigger publishing
- The `justfile` is the authoritative source for dev commands — prefer `just <cmd>` over raw cargo

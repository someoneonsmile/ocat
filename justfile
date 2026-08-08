# ocat — opencode session viewer（通过 pager 查看 opencode session）
# https://github.com/cargo/just

_default:
    @just --list

# 快速检查（类型检查，不生成二进制）
check:
    cargo check

# 编译 debug 版本
build:
    SHELL_HELP_DIR=shell_help cargo build

# 编译 release 版本（优化）
release:
    SHELL_HELP_DIR=shell_help cargo build --release

# 运行（debug）
run *args:
    cargo run -- {{args}}

# 运行（release）
run-release *args:
    cargo run --release -- {{args}}

# 运行测试
test:
    cargo test

# 格式化代码
fmt:
    cargo fmt

# 检查格式（不修改）
fmt-check:
    cargo fmt --check

# Clippy lint
lint:
    cargo clippy

# Clippy lint（严格模式，warning 视为 error）
lint-strict:
    cargo clippy -- -D warnings

# Clippy 自动修复
fix:
    cargo clippy --fix --allow-dirty --allow-staged

# Clippy 自动修复 + 严格 lint 验证
fix-strict: fix lint-strict

# 清理构建产物
clean:
    cargo clean
    rm -rf {{ SHELL_HELP_DIR }}

# 安装到系统（/usr/local/bin/ocat + shell 补全 + man）


DESTDIR := ""
PREFIX := "/usr/local"
BINDIR := PREFIX / "bin"
DATADIR := PREFIX / "share"
MANDIR := DATADIR / "man"
SHELL_HELP_DIR := "shell_help"

install:
    SHELL_HELP_DIR={{ SHELL_HELP_DIR }} cargo build --release
    sudo install -Dm755 target/release/ocat "{{ DESTDIR }}{{ BINDIR }}/ocat"
    sudo install -Dm644 {{ SHELL_HELP_DIR }}/complete/ocat.bash "{{ DESTDIR }}{{ DATADIR }}/bash-completion/completions/ocat"
    sudo install -Dm644 {{ SHELL_HELP_DIR }}/complete/_ocat "{{ DESTDIR }}{{ DATADIR }}/zsh/site-functions/_ocat"
    sudo install -Dm644 {{ SHELL_HELP_DIR }}/complete/ocat.fish "{{ DESTDIR }}{{ DATADIR }}/fish/vendor_completions.d/ocat.fish"
    sudo install -Dm644 {{ SHELL_HELP_DIR }}/man/ocat.1 "{{ DESTDIR }}{{ MANDIR }}/man1/ocat.1"

# 从系统中卸载 ocat 及其补全和 man 手册页
uninstall:
    sudo rm -f "{{ DESTDIR }}{{ BINDIR }}/ocat"
    sudo rm -f "{{ DESTDIR }}{{ DATADIR }}/bash-completion/completions/ocat"
    sudo rm -f "{{ DESTDIR }}{{ DATADIR }}/zsh/site-functions/_ocat"
    sudo rm -f "{{ DESTDIR }}{{ DATADIR }}/fish/vendor_completions.d/ocat.fish"
    sudo rm -f "{{ DESTDIR }}{{ MANDIR }}/man1/ocat.1"

# 监听文件变更，自动 check
watch:
    cargo watch -x check

# 监听文件变更，自动 fmt + lint + build
watch-all:
    cargo watch -x fmt -x clippy -x build

# 全量检查（fmt → clippy → test → release build）
ci: fmt-check lint-strict test release

# ─── AUR ──────────────────────────────────────────────

# 重新生成 .SRCINFO（手动修改 PKGBUILD 后执行）
aur-srcinfo:
    cd aur && makepkg --printsrcinfo > .SRCINFO

# 提交 aur/ 变更并推送到 AUR
aur-push:
    git add aur/
    git commit -m "chore: update AUR package"
    git subtree push --prefix=aur aur master

# 发布新版本到 AUR（自动更新版本号 + 推送）
aur-release VERSION:
    sed -i 's/^pkgver=.*/pkgver={{VERSION}}/' aur/PKGBUILD
    sed -i 's/^pkgrel=.*/pkgrel=1/' aur/PKGBUILD
    cd aur && makepkg --printsrcinfo > .SRCINFO
    git add aur/
    git commit -m "chore: update AUR to v{{VERSION}}"
    git subtree push --prefix=aur aur master

# ─── Nightly AUR ───────────────────────────────────────

# 重新生成 aur-nightly/.SRCINFO
aur-nightly-srcinfo:
    cd aur-nightly && makepkg --printsrcinfo > .SRCINFO

# 提交 aur-nightly/ 变更并推送到 Nightly AUR
aur-nightly-push:
    git add aur-nightly/
    git commit -m "chore: update nightly AUR package"
    git subtree push --prefix=aur-nightly aur-nightly master

# 发布 Nightly AUR（自动使用当前日期作为版本号 + 推送）
aur-nightly-release:
    #!/usr/bin/env bash
    set -euo pipefail
    DATE=$(date +%Y%m%d)
    sed -i "s/^pkgver=.*/pkgver=$DATE/" aur-nightly/PKGBUILD
    sed -i 's/^pkgrel=.*/pkgrel=1/' aur-nightly/PKGBUILD
    cd aur-nightly && makepkg --printsrcinfo > .SRCINFO
    git add aur-nightly/
    git commit -m "chore: update nightly AUR to $DATE"
    git subtree push --prefix=aur-nightly aur-nightly master

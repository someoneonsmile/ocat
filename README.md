# ocat — opencode session viewer

将 `opencode export` 导出的 JSON 格式 session 数据渲染为人类可读的文本，通过 pager（less）展示，获得类似 `git log` / `git diff` 的翻阅体验。

## 动机

opencode 的 session 回放功能（`opencode export <id>`）输出的是原始 JSON，直接阅读体验很差。而 opencode 内置的 session 查看界面不支持键盘翻页（`j`/`k`、`Ctrl-D`/`Ctrl-U` 等），翻阅长 session 极其痛苦。

`ocat` 解决这个问题——它接受 `opencode export` 的 JSON 输出，渲染成结构清晰的文本格式，然后交给 less（或其他 pager）展示。

## 使用方式

```bash
# 基本用法：管道输入
opencode export <session-id> | ocat

# 等效写法
ocat <(opencode export <session-id>)

# 指定 pager
opencode export <session-id> | ocat --pager bat

# 不通过 pager，直接打印到 stdout
opencode export <session-id> | ocat --no-pager

# 也可以从文件读取
ocat session.json
```

## 效果要求

### 渲染格式

session JSON 中包含的信息按以下层次渲染：

1. **Session 元信息** — session ID、时间段、消息数量、agent 使用情况等
2. **消息流** — 按时间顺序展示每一条消息，包括：
   - 角色标识（user / assistant / system）
   - 时间戳
   - 消息内容（Markdown 渲染为带 ANSI 颜色的终端文本）
3. **工具调用** — 折叠/展开显示工具调用和返回结果
4. **思考过程（thinking）** — 默认折叠，可按需展开

### Pager 行为

- 默认使用 `less -R`（`-R` 启用 ANSI 颜色解析）
- 已在 `$PAGER` 环境变量 → 使用该值
- `--pager <name>` 显式指定
- `--no-pager` 直接输出到 stdout
- 内部直接 fork+exec pager 进程（类似 git 的做法），不依赖中间缓冲

### 视觉效果

- 用 ANSI 颜色区分角色：用户消息、助手消息、系统消息、工具调用
- 用分隔线区分不同消息
- 折叠长输出（工具调用结果、thinking 内容），用 `…` 占位提示
- 终端宽度自适应

### 输入格式兼容

- 支持 `opencode export` 的所有输出格式版本（向后兼容）
- 输入格式不合法时给出明确报错
- 支持 stdin 管道输入和文件路径参数两种方式

## 技术栈

- **语言**：Rust
- **CLI 参数解析**：clap (derive)
- **JSON 解析**：serde + serde_json
- **ANSI 渲染**：termcolor / anstyle
- **Pager 调用**：pager crate 或 fork+exec less 直接调用
- **Shell 补全**：clap_complete（构建时生成）
- **Man 手册**：clap_mangen（构建时生成）
- **CI/CD**：GitHub Actions（参考 deref 项目模式）
- **AUR**：ocat-bin（稳定版）+ ocat-nightly-bin（每夜构建）

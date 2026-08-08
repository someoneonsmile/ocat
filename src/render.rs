use std::io::Write;

use chrono::Local;
use chrono::TimeZone;
use termcolor::{Buffer, Color, ColorSpec, WriteColor};
use termimad::MadSkin;

use crate::types::*;

const HLINE: &str = "──────────────────────────────────────────────────────────────────────";

pub fn render_session(json: &str) -> anyhow::Result<String> {
    let session: Session = serde_json::from_str(json)?;
    let mut buf = Buf::new();
    let msg_count = session.messages.len();

    render_meta(&mut buf, &session.info, msg_count);
    buf.writeln("");
    render_messages(&mut buf, &session.messages);

    Ok(buf.into_string())
}

#[allow(dead_code)]
struct Colors {
    dim: ColorSpec,
    bold: ColorSpec,
    reset: ColorSpec,
    cyan_bold: ColorSpec,
}

impl Colors {
    fn new() -> Self {
        let dim = with_style(ColorSpec::new(), Style::Dim);
        let bold = with_style(ColorSpec::new(), Style::Bold);
        let reset = ColorSpec::new().clone();
        let cyan_bold = with_fg_bold(ColorSpec::new(), Color::Cyan);
        Self {
            dim,
            bold,
            reset,
            cyan_bold,
        }
    }
}

struct Buf {
    inner: Buffer,
    c: Colors,
}

enum Style {
    Dim,
    Bold,
}

fn with_style(mut spec: ColorSpec, style: Style) -> ColorSpec {
    match style {
        Style::Dim => spec.set_dimmed(true),
        Style::Bold => spec.set_bold(true),
    };
    spec
}

fn with_fg(mut spec: ColorSpec, color: Color) -> ColorSpec {
    spec.set_fg(Some(color));
    spec
}

fn with_fg_bold(spec: ColorSpec, color: Color) -> ColorSpec {
    with_style(with_fg(spec, color), Style::Bold)
}

impl Buf {
    fn new() -> Self {
        Self {
            inner: Buffer::ansi(),
            c: Colors::new(),
        }
    }

    fn write(&mut self, spec: &ColorSpec, text: &str) {
        self.inner.set_color(spec).ok();
        write!(self.inner, "{text}").ok();
        self.inner.reset().ok();
    }

    #[allow(dead_code)]
    fn write_dim(&mut self, text: &str) {
        self.write(&self.c.dim.clone(), text);
    }
    fn write_bold(&mut self, text: &str) {
        self.write(&self.c.bold.clone(), text);
    }
    fn write_reset(&mut self, text: &str) {
        self.write(&self.c.reset.clone(), text);
    }
    fn write_cyan_bold(&mut self, text: &str) {
        self.write(&self.c.cyan_bold.clone(), text);
    }

    fn write_fg(&mut self, color: Color, text: &str) {
        self.write(&with_fg(ColorSpec::new(), color), text);
    }
    fn write_fg_bold(&mut self, color: Color, text: &str) {
        self.write(&with_fg_bold(ColorSpec::new(), color), text);
    }

    fn writeln(&mut self, text: &str) {
        self.write_reset(&format!("{text}\n"));
    }

    /// 直接写入原始字节（保留已有的 ANSI 转义序列），用于 termimad 渲染后的文本。
    fn write_raw(&mut self, text: &str) {
        write!(self.inner, "{text}").ok();
    }

    fn into_string(self) -> String {
        String::from_utf8(self.inner.into_inner()).unwrap_or_default()
    }
}

fn render_meta(buf: &mut Buf, info: &SessionInfo, msg_count: usize) {
    let created = fmt_ts(info.time.created);
    let updated = fmt_ts(info.time.updated);
    let duration = duration_str(info.time.created, info.time.updated);
    let total_tokens = info.tokens.input + info.tokens.output + info.tokens.reasoning;

    buf.write_cyan_bold("  Session:");
    buf.write_reset(&format!(" {}\n", info.id));
    buf.write_cyan_bold("  Title:");
    buf.write_reset(&format!("   {}\n", info.title));
    buf.write_cyan_bold("  Agent:");
    buf.write_reset(&format!("   {}\n", info.agent));
    buf.write_cyan_bold("  Model:");
    buf.write_reset(&format!(
        "   {} ({})\n",
        info.model.id, info.model.provider_id
    ));
    buf.write_cyan_bold("  Time:");
    buf.write_reset(&format!("    {created} → {updated} ({duration})\n"));
    buf.write_cyan_bold("  Tokens:");
    buf.write_reset(&format!("  {} ", fmt_num(total_tokens)));
    buf.write_dim(&format!(
        "(in:{} + out:{} + rsn:{})",
        fmt_num(info.tokens.input),
        fmt_num(info.tokens.output),
        fmt_num(info.tokens.reasoning),
    ));
    buf.writeln("");
    buf.write_cyan_bold("  Cost:");
    buf.write_reset(&format!("    ${:.4}\n", info.cost));
    buf.write_cyan_bold("  Msgs:");
    buf.write_reset(&format!("    {msg_count}\n"));
}

fn render_messages(buf: &mut Buf, msgs: &[Message]) {
    if msgs.is_empty() {
        return;
    }

    buf.write_bold(&format!("  Messages ({})\n", msgs.len()));
    buf.write_dim(HLINE);
    buf.writeln("");

    for msg in msgs {
        buf.writeln("");
        render_message(buf, msg);
    }
}

fn render_message(buf: &mut Buf, msg: &Message) {
    let role = &msg.info.role;
    let (role_color, role_label) = match role.as_str() {
        "user" => (Color::Green, "USER"),
        "assistant" => (Color::Blue, "ASSISTANT"),
        "system" => (Color::Yellow, "SYSTEM"),
        _ => (Color::White, "OTHER"),
    };

    let created = fmt_ts(msg.info.time.created);
    let duration = msg
        .info
        .time
        .completed
        .filter(|&end| end > msg.info.time.created)
        .map(|end| {
            format!(
                " → {} ({})",
                fmt_time_only(end),
                duration_str(msg.info.time.created, end)
            )
        })
        .unwrap_or_default();

    let finish = msg.info.finish.as_deref().unwrap_or("");
    let finish_str = if finish.is_empty() {
        String::new()
    } else {
        format!(" · {finish}")
    };

    let tokens_str = msg
        .info
        .tokens
        .as_ref()
        .map(|t| {
            let total = t.input + t.output + t.reasoning;
            format!(" · {} tokens", fmt_num(total))
        })
        .unwrap_or_default();

    let cost_str = msg
        .info
        .cost
        .map(|c| format!(" · ${c:.4}"))
        .unwrap_or_default();

    buf.write_fg_bold(role_color, &format!("┌─ {role_label}"));
    buf.write_reset(" ");
    buf.write_dim(&format!(
        "{created}{duration}{finish_str}{tokens_str}{cost_str}"
    ));
    buf.writeln("");

    for part in &msg.parts {
        render_part(buf, part);
    }

    buf.write_dim("└─");
    buf.writeln("");
}

fn render_part(buf: &mut Buf, part: &Part) {
    match part {
        Part::Text { text, .. } => {
            let skin = MadSkin::default();
            // 宽度 78 = 终端 80 列 - 前缀 "│ " 占 2 列
            let rendered = skin.text(text, Some(78));
            for line in rendered.to_string().lines() {
                buf.write_dim("│");
                buf.write_raw(&format!(" {line}\n"));
            }
        }
        Part::Reasoning { text, .. } => {
            let preview_lines: Vec<&str> = text.lines().take(5).collect();
            let total_lines = text.lines().count();
            buf.write_dim("│");
            buf.write_reset(" ");
            buf.write_fg(Color::Magenta, "💭 Reasoning:");
            buf.writeln("");
            for line in &preview_lines {
                let chars: Vec<char> = line.chars().collect();
                let truncated = chars.len() > 200;
                let preview: String = chars.iter().take(200).collect();
                let suffix = if truncated { "…" } else { "" };
                buf.write_dim("│");
                buf.write_reset("  ");
                buf.write_dim(&format!("{preview}{suffix}"));
                buf.writeln("");
            }
            if total_lines > 5 {
                buf.write_dim("│");
                buf.write_reset("  ");
                buf.write_dim(&format!("… ({} more lines)", total_lines - 5));
                buf.writeln("");
            }
        }
        Part::Tool {
            tool,
            call_id: _call_id,
            state,
        } => {
            let desc = state
                .input
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(tool);
            let cmd = state
                .input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let status_color = match state.status.as_str() {
                "completed" => Color::Green,
                "error" => Color::Red,
                _ => Color::Yellow,
            };

            buf.write_dim("│");
            buf.write_reset(" ");
            buf.write_fg(Color::Yellow, &format!("🔧 {tool}"));
            buf.write_dim(" · ");
            buf.write_fg(Color::Cyan, desc);
            buf.write_dim(" · ");
            buf.write_fg(status_color, &state.status);
            buf.writeln("");

            if !cmd.is_empty() {
                buf.write_dim("│");
                buf.write_reset(" ");
                buf.write_dim(&format!("  $ {cmd}"));
                buf.writeln("");
            }

            if let serde_json::Value::Object(ref input_map) = state.input {
                let params: Vec<String> = input_map
                    .iter()
                    .filter(|(k, _)| k.as_str() != "description" && k.as_str() != "command")
                    .filter_map(|(k, v)| match v {
                        serde_json::Value::String(s) => Some(format!("{k}: {s}")),
                        serde_json::Value::Number(n) => Some(format!("{k}: {n}")),
                        serde_json::Value::Bool(b) => Some(format!("{k}: {b}")),
                        _ => None,
                    })
                    .collect();
                if !params.is_empty() {
                    buf.write_dim("│");
                    buf.write_reset(" ");
                    buf.write_dim(&format!("  {}", params.join(", ")));
                    buf.writeln("");
                }
            }

            if let Some(output) = &state.output {
                let lines: Vec<&str> = output.lines().take(1).collect();
                let total = output.lines().count();
                for line in &lines {
                    buf.write_dim("│");
                    buf.write_reset(" ");
                    buf.write_dim(&format!("  | {line}"));
                    buf.writeln("");
                }
                if total > 1 {
                    buf.write_dim("│");
                    buf.write_reset(" ");
                    buf.write_dim(&format!("  … ({} more lines)", total - 1));
                    buf.writeln("");
                }
            }
        }
        Part::StepStart => {
            buf.write_dim("│");
            buf.write_reset(" ");
            buf.write_dim("▶ step-start");
            buf.writeln("");
        }
        Part::StepFinish {
            reason,
            tokens,
            cost,
        } => {
            let r = reason.as_deref().unwrap_or("");
            let t = tokens
                .as_ref()
                .map(|t| {
                    let total = t.input + t.output + t.reasoning;
                    format!(" · {total} tokens", total = fmt_num(total))
                })
                .unwrap_or_default();
            let c = cost.map(|c| format!(" · ${c:.4}")).unwrap_or_default();
            buf.write_dim("│");
            buf.write_reset(" ");
            buf.write_dim(&format!("◼ step-finish · {r}{t}{c}"));
            buf.writeln("");
        }
        Part::Other(_) => {}
    }
}

fn fmt_ts(ms: i64) -> String {
    match Local.timestamp_millis_opt(ms) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => ms.to_string(),
    }
}

fn fmt_time_only(ms: i64) -> String {
    match Local.timestamp_millis_opt(ms) {
        chrono::LocalResult::Single(dt) => dt.format("%H:%M:%S").to_string(),
        _ => ms.to_string(),
    }
}

fn duration_str(start_ms: i64, end_ms: i64) -> String {
    let delta = end_ms - start_ms;
    if delta < 1000 {
        format!("{delta}ms")
    } else if delta < 60_000 {
        format!("{:.1}s", delta as f64 / 1000.0)
    } else if delta < 3_600_000 {
        let m = delta / 60_000;
        let s = (delta % 60_000) / 1000;
        format!("{m}m{s}s")
    } else {
        let h = delta / 3_600_000;
        let m = (delta % 3_600_000) / 60_000;
        format!("{h}h{m}m")
    }
}

fn fmt_num(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_str() {
        assert_eq!(duration_str(0, 500), "500ms");
        assert_eq!(duration_str(0, 2_500), "2.5s");
        assert_eq!(duration_str(0, 65_000), "1m5s");
        assert_eq!(duration_str(0, 3_600_000), "1h0m");
    }

    #[test]
    fn test_fmt_num() {
        assert_eq!(fmt_num(42), "42");
        assert_eq!(fmt_num(1_500), "1.5K");
        assert_eq!(fmt_num(2_000_000), "2.0M");
    }

    #[test]
    fn test_render_single_message() {
        let json = r#"{
            "info": {
                "id": "ses_test",
                "slug": "test",
                "title": "Test Session",
                "agent": "Sisyphus",
                "model": { "id": "test-model", "providerID": "test-provider" },
                "version": "1.0.0",
                "cost": 0.001,
                "tokens": { "input": 10, "output": 5, "reasoning": 2, "cache": { "read": 0, "write": 0 } },
                "time": { "created": 1782116223309, "updated": 1782116230544 }
            },
            "messages": [
                {
                    "info": {
                        "role": "user",
                        "time": { "created": 1782116223330 },
                        "agent": null,
                        "model": null,
                        "finish": null,
                        "tokens": null,
                        "cost": null,
                        "id": null,
                        "summary": { "diffs": [] }
                    },
                    "parts": [
                        { "type": "text", "text": "hello world" }
                    ]
                },
                {
                    "info": {
                        "role": "assistant",
                        "time": { "created": 1782116223349, "completed": 1782116230534 },
                        "agent": null,
                        "model": null,
                        "finish": "stop",
                        "tokens": { "input": 10, "output": 5, "reasoning": 2, "cache": { "read": 0, "write": 0 } },
                        "cost": 0.001,
                        "id": null,
                        "summary": { "diffs": [] }
                    },
                    "parts": [
                        { "type": "reasoning", "text": "thinking..." },
                        { "type": "text", "text": "hi there" }
                    ]
                }
            ]
        }"#;

        let result = render_session(json).unwrap();
        assert!(result.contains("ses_test"));
        assert!(result.contains("Test Session"));
        assert!(result.contains("USER"));
        assert!(result.contains("hello world"));
        assert!(result.contains("ASSISTANT"));
        assert!(result.contains("hi there"));
        assert!(result.contains("Reasoning"));
    }
}

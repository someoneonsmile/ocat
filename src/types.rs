// session JSON 的 serde 结构体定义
// 对应 `opencode export <id>` 的输出格式
#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Session {
    pub info: SessionInfo,
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    #[allow(dead_code)]
    pub slug: String,
    pub title: String,
    pub agent: String,
    pub model: ModelInfo,
    #[allow(dead_code)]
    pub version: String,
    pub cost: f64,
    pub tokens: TokenStats,
    pub time: TimeRange,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(rename = "providerID")]
    pub provider_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenStats {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache: CacheStats,
}

#[derive(Debug, Deserialize)]
pub struct CacheStats {
    pub read: u64,
    pub write: u64,
}

#[derive(Debug, Deserialize)]
pub struct TimeRange {
    pub created: i64,
    pub updated: i64,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub info: MessageInfo,
    pub parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
pub struct MessageInfo {
    pub role: String,
    pub time: MessageTime,
    pub agent: Option<String>,
    pub model: Option<MsgModelInfo>,
    pub finish: Option<String>,
    pub tokens: Option<TokenStats>,
    pub cost: Option<f64>,
    pub id: Option<String>,
    pub summary: Option<Summary>,
}

#[derive(Debug, Deserialize)]
pub struct MsgModelInfo {
    #[serde(rename = "providerID")]
    pub provider_id: Option<String>,
    #[serde(rename = "modelID")]
    pub model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessageTime {
    pub created: i64,
    pub completed: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Summary {
    #[allow(dead_code)]
    pub diffs: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Part {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(default)]
        time: Option<PartTime>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        text: String,
        #[serde(default)]
        time: Option<PartTime>,
    },
    #[serde(rename = "tool")]
    Tool {
        tool: String,
        #[serde(rename = "callID")]
        call_id: Option<String>,
        state: ToolState,
    },
    #[serde(rename = "step-start")]
    StepStart,
    #[serde(rename = "step-finish")]
    StepFinish {
        reason: Option<String>,
        tokens: Option<TokenStats>,
        cost: Option<f64>,
    },
    #[serde(untagged)]
    Other(serde_json::Value),
}

#[derive(Debug, Deserialize)]
pub struct PartTime {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ToolState {
    pub status: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    #[allow(dead_code)]
    pub title: Option<String>,
    #[allow(dead_code)]
    pub metadata: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub time: Option<PartTime>,
}

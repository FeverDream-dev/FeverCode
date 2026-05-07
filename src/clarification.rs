use crate::providers::{ChatMessage, ChatRequest, MessageRole, Provider};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ClarificationSession {
    pub original_request: String,
    pub questions: Vec<String>,
    pub answers: Vec<String>,
    pub ready: bool,
}

impl ClarificationSession {
    pub fn new(request: String) -> Self {
        Self {
            original_request: request,
            questions: Vec::new(),
            answers: Vec::new(),
            ready: false,
        }
    }

    pub fn needs_answers(&self) -> bool {
        !self.questions.is_empty() && self.answers.len() < self.questions.len()
    }

    pub fn current_question(&self) -> Option<&str> {
        self.questions.get(self.answers.len()).map(|s| s.as_str())
    }

    pub fn push_answer(&mut self, answer: String) {
        self.answers.push(answer);
    }

    pub fn all_answered(&self) -> bool {
        !self.questions.is_empty() && self.answers.len() == self.questions.len()
    }

    pub fn summary(&self) -> String {
        let mut out = format!("Original request: {}\n", self.original_request);
        for (i, (q, a)) in self.questions.iter().zip(self.answers.iter()).enumerate() {
            out.push_str(&format!("Q{}: {}\nA{}: {}\n", i + 1, q, i + 1, a));
        }
        out
    }
}

pub fn is_vague_request(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.len() < 40 {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    let has_tech = [
        "rust",
        "python",
        "js",
        "typescript",
        "go",
        "java",
        "c++",
        "react",
        "vue",
        "angular",
        "svelte",
        "solid",
        "next",
        "nuxt",
        "node",
        "bun",
        "deno",
        "flask",
        "django",
        "rails",
        "spring",
        "laravel",
        "express",
        "actix",
        "rocket",
        "axum",
        "tauri",
        "electron",
        "flutter",
        "swift",
        "kotlin",
        "postgres",
        "mysql",
        "mongo",
        "redis",
        "sqlite",
        "docker",
        "k8s",
    ]
    .iter()
    .any(|kw| lower.contains(kw));
    let has_action = [
        "build",
        "create",
        "fix",
        "refactor",
        "add",
        "implement",
        "write",
        "delete",
        "update",
        "change",
        "modify",
        "rewrite",
        "convert",
        "migrate",
        "integrate",
        "setup",
        "configure",
        "deploy",
        "test",
    ]
    .iter()
    .any(|kw| lower.contains(kw));
    let has_target = lower.contains(".")
        || lower.contains('/')
        || lower.contains("file")
        || lower.contains("folder");
    !has_tech || !has_action || !has_target
}

fn clarifier_system_prompt() -> String {
    "You are a senior requirements analyst. The user gave a coding request that may be incomplete or vague.\n\
    Your job is to ask 1-3 short clarifying questions that will help a developer understand EXACTLY what to build.\n\
    Rules:\n\
    - Only ask about missing CRITICAL details (tech stack, scope, file paths, expected behavior, constraints).\n\
    - Do NOT ask questions already answered in the request.\n\
    - If the request is already completely clear, respond with exactly: READY\n\
    - Format each question on its own line starting with Q: ".to_string()
}

pub async fn generate_questions(
    provider: &dyn Provider,
    model: &str,
    request: &str,
) -> Result<Vec<String>> {
    let req = ChatRequest {
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: clarifier_system_prompt(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: format!("Request: {}", request),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        model: Some(model.to_string()),
        tools: None,
        temperature: Some(0.2),
        max_tokens: Some(300),
    };
    let resp = provider.chat_with_tools(req).await?;
    let text = resp.content.unwrap_or_default();
    if text.trim().eq_ignore_ascii_case("READY") {
        return Ok(Vec::new());
    }
    let questions: Vec<String> = text
        .lines()
        .filter(|l| l.trim().starts_with("Q:"))
        .map(|l| l.trim().strip_prefix("Q:").unwrap_or(l).trim().to_string())
        .collect();
    if questions.is_empty() {
        // fallback: treat each non-empty line as a question
        Ok(text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect())
    } else {
        Ok(questions)
    }
}

fn readiness_system_prompt() -> String {
    "You are a senior architect. Given the original request and clarification answers, determine if you have 100% certainty about what to build.\n\
    Respond ONLY with a JSON object in this exact format (no markdown, no prose):\n\
    {\"certainty\": 0-100, \"ready\": true/false, \"missing_info\": \"short note or empty string\"}".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReadinessResult {
    pub certainty: u8,
    pub ready: bool,
    #[serde(default)]
    pub missing_info: String,
}

pub async fn check_readiness(
    provider: &dyn Provider,
    model: &str,
    session: &ClarificationSession,
) -> Result<ReadinessResult> {
    let req = ChatRequest {
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: readiness_system_prompt(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: session.summary(),
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        model: Some(model.to_string()),
        tools: None,
        temperature: Some(0.0),
        max_tokens: Some(200),
    };
    let resp = provider.chat_with_tools(req).await?;
    let text = resp.content.unwrap_or_default();
    let json_text = crate::agent_loop::strip_markdown_code_blocks(&text);
    let result: ReadinessResult = serde_json::from_str(&json_text).unwrap_or(ReadinessResult {
        certainty: 0,
        ready: false,
        missing_info: text,
    });
    Ok(result)
}

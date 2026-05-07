use anyhow::Result;
use crate::providers::{ChatMessage, ChatRequest, MessageRole, Provider};
use crate::rag::store::VectorStore;
use crate::rag::embedder::Embedder;

const MAX_ITERATIONS: usize = 6;
const TOP_K: usize = 5;

#[derive(Debug, Clone)]
pub struct MastermindResult {
    pub answer: String,
    pub iterations: usize,
    pub sources: Vec<String>,
    pub queries: Vec<String>,
}

fn system_prompt() -> String {
    "You are the Local Mastermind — a reasoning engine powered by a small local LLM with access to a local document database.\n\
    Your strategy: retrieve relevant documents, reason over them, and iteratively refine your search until you have enough information to answer with high confidence.\n\
    Rules:\n\
    1. If the user asks something that requires factual knowledge NOT in the retrieved documents, generate search queries to find it.\n\
    2. After reading retrieved chunks, produce a partial answer and decide if more info is needed.\n\
    3. Respond with ONLY one of these formats (no markdown fences, no extra prose):\n\
       SEARCH: query text here\n\
       ANSWER: your final answer here\n\
       PARTIAL: your reasoning so far\n\
    4. If you have enough info, always end with ANSWER:.\n\
    5. Cite sources when possible.".to_string()
}

pub async fn run(
    provider: &dyn Provider,
    model: &str,
    store: &VectorStore,
    embedder: &dyn Embedder,
    user_query: &str,
) -> Result<MastermindResult> {
    let mut queries = vec![user_query.to_string()];
    let mut context = String::new();
    let mut sources_seen = std::collections::HashSet::new();
    let mut iterations = 0usize;

    for _ in 0..MAX_ITERATIONS {
        iterations += 1;
        let query = queries.last().unwrap();
        let embedding = embedder.embed(query).await?;
        let results = store.search(&embedding, TOP_K);

        if results.is_empty() {
            context.push_str(&format!("\n[Iteration {}] No relevant documents found for: {}\n", iterations, query));
        } else {
            context.push_str(&format!("\n[Iteration {}] Retrieved {} chunks:\n", iterations, results.len()));
            for (chunk, score) in &results {
                if sources_seen.insert(chunk.source.clone()) {
                    context.push_str(&format!(
                        "Source: {} (score: {:.3})\n{}\n---\n",
                        chunk.source, score, chunk.text
                    ));
                }
            }
        }

        let prompt = format!(
            "User query: {}\n\nRetrieved context so far:{}\n\n\
            If you need more information, respond with:\nSEARCH: <your follow-up search query>\n\
            If you have enough information, respond with:\nANSWER: <your final answer>\n\
            If you are reasoning but not ready, respond with:\nPARTIAL: <your reasoning>",
            user_query, context
        );

        let req = ChatRequest {
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: system_prompt(),
                    tool_calls: None,
                    tool_call_id: None,
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: prompt,
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
            model: Some(model.to_string()),
            tools: None,
            temperature: Some(0.2),
            max_tokens: Some(800),
        };

        let resp = provider.chat_with_tools(req).await?;
        let text = resp.content.unwrap_or_default();
        let line = text.lines().next().unwrap_or("")
            .trim()
            .to_string();

        if line.starts_with("ANSWER:") {
            let answer = line.strip_prefix("ANSWER:").unwrap_or("")
                .trim()
                .to_string();
            return Ok(MastermindResult {
                answer,
                iterations,
                sources: sources_seen.into_iter().collect(),
                queries: queries.clone(),
            });
        } else if line.starts_with("SEARCH:") {
            let new_query = line.strip_prefix("SEARCH:").unwrap_or("")
                .trim()
                .to_string();
            if new_query.is_empty() || queries.contains(&new_query) {
                // Prevent loops
                break;
            }
            queries.push(new_query);
        } else if line.starts_with("PARTIAL:") {
            // Continue to next iteration with refined reasoning
            continue;
        } else {
            // Unstructured response — treat as final answer if we've done enough iterations
            if iterations >= 3 {
                return Ok(MastermindResult {
                    answer: text.trim().to_string(),
                    iterations,
                    sources: sources_seen.into_iter().collect(),
                    queries: queries.clone(),
                });
            }
        }
    }

    // Max iterations reached — synthesize best-effort answer
    let final_prompt = format!(
        "User query: {}\n\nAll retrieved context:{}\n\n\
        Provide the best possible answer based on the above context. If insufficient, say so clearly.",
        user_query, context
    );
    let req = ChatRequest {
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: final_prompt,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        model: Some(model.to_string()),
        tools: None,
        temperature: Some(0.2),
        max_tokens: Some(1200),
    };
    let resp = provider.chat_with_tools(req).await?;
    let answer = resp.content.unwrap_or_default().trim().to_string();

    Ok(MastermindResult {
        answer,
        iterations,
        sources: sources_seen.into_iter().collect(),
        queries: queries.clone(),
    })
}

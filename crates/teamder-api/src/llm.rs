use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teamder_core::{error::TeamderError, models::peer_review::ReviewScores};

#[derive(Clone)]
pub struct ReviewLlmClient {
    http: Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQa {
    pub question: String,
    pub answer: String,
}

pub struct ReviewAssistContext<'a> {
    pub reviewer_name: &'a str,
    pub reviewee_name: &'a str,
    pub project_name: &'a str,
    pub scores: ReviewScores,
    pub initial_body: &'a str,
    pub answers: &'a [ReviewQa],
    pub clarification_note: Option<&'a str>,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u16,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
}

impl ReviewLlmClient {
    pub fn from_env() -> Self {
        let base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "http://1.34.172.117:8000".to_string())
            .trim_end_matches('/')
            .to_string();
        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "Qwen3.6-27B-MTP-Q4".to_string());
        let api_key = std::env::var("LLM_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let timeout_secs = std::env::var("LLM_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(45);
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("failed to build LLM HTTP client");

        Self {
            http,
            base_url,
            model,
            api_key,
        }
    }

    pub async fn clarification_questions(
        &self,
        ctx: ReviewAssistContext<'_>,
    ) -> Result<Vec<String>, TeamderError> {
        let prompt = format!(
            "{}\n\nTask: Ask 2 or 3 concise follow-up questions that would make this peer review clearer, fairer, and more specific. Do not ask about information already answered. If the commenter says part of the preview was misleading, focus on that ambiguity.\n\nReturn only JSON shaped exactly like {{\"questions\":[\"...\"]}}.",
            render_context(&ctx)
        );
        let content = self.complete(prompt, 450).await?;
        parse_questions(&content)
    }

    pub async fn summarize_review(
        &self,
        ctx: ReviewAssistContext<'_>,
    ) -> Result<String, TeamderError> {
        let prompt = format!(
            "{}\n\nTask: Write the final peer review body as a preview for the commenter. Use only the supplied facts. Keep it constructive, specific, and balanced. Preserve the commenter's language when possible. Avoid inventing achievements, private details, or exaggerated praise. Target 70 to 130 words.\n\nReturn only JSON shaped exactly like {{\"summary\":\"...\"}}.",
            render_context(&ctx)
        );
        let content = self.complete(prompt, 700).await?;
        parse_summary(&content)
    }

    async fn complete(&self, user_prompt: String, max_tokens: u16) -> Result<String, TeamderError> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: "You help Teamder commenters write peer reviews. You ask useful clarification questions and summarize only user-provided facts. Output valid JSON only, with no markdown.".to_string(),
                },
                ChatMessage { role: "user", content: user_prompt },
            ],
            temperature: 0.2,
            max_tokens,
            stream: false,
        };

        let mut req = self.http.post(url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }

        let res = req
            .send()
            .await
            .map_err(|e| TeamderError::Internal(format!("LLM request failed: {e}")))?;
        let status = res.status();
        if !status.is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(TeamderError::Internal(format!(
                "LLM request failed with status {status}: {text}"
            )));
        }

        let data: ChatCompletionResponse = res
            .json()
            .await
            .map_err(|e| TeamderError::Internal(format!("LLM response was not valid JSON: {e}")))?;
        data.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| TeamderError::Internal("LLM returned no choices".into()))
    }
}

fn render_context(ctx: &ReviewAssistContext<'_>) -> String {
    let mut out = format!(
        "Peer review context:\nReviewer: {}\nReviewee: {}\nContext: {}\nScores: skill {}/5, communication {}/5, reliability {}/5, teamwork {}/5\nInitial comment:\n{}",
        ctx.reviewer_name,
        ctx.reviewee_name,
        ctx.project_name,
        ctx.scores.skill,
        ctx.scores.communication,
        ctx.scores.reliability,
        ctx.scores.teamwork,
        ctx.initial_body.trim()
    );
    if !ctx.answers.is_empty() {
        out.push_str("\n\nClarification answers:");
        for qa in ctx.answers {
            out.push_str(&format!(
                "\nQ: {}\nA: {}",
                qa.question.trim(),
                qa.answer.trim()
            ));
        }
    }
    if let Some(note) = ctx.clarification_note {
        if !note.trim().is_empty() {
            out.push_str(&format!(
                "\n\nMisleading or unclear preview part:\n{}",
                note.trim()
            ));
        }
    }
    out
}

fn parse_questions(content: &str) -> Result<Vec<String>, TeamderError> {
    let value = parse_json_value(content)?;
    let questions = value
        .get("questions")
        .and_then(Value::as_array)
        .ok_or_else(|| TeamderError::Internal("LLM did not return questions".into()))?;

    let out: Vec<String> = questions
        .iter()
        .filter_map(|q| {
            q.as_str().map(str::to_string).or_else(|| {
                q.get("question")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
        })
        .map(|q| q.trim().to_string())
        .filter(|q| !q.is_empty())
        .take(3)
        .collect();

    if out.len() < 2 {
        return Err(TeamderError::Internal(
            "LLM returned too few questions".into(),
        ));
    }
    Ok(out)
}

fn parse_summary(content: &str) -> Result<String, TeamderError> {
    let value = parse_json_value(content)?;
    let summary = value
        .get("summary")
        .or_else(|| value.get("review"))
        .or_else(|| value.get("comment"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| TeamderError::Internal("LLM did not return a summary".into()))?;
    Ok(summary.to_string())
}

fn parse_json_value(content: &str) -> Result<Value, TeamderError> {
    let after_thinking = content
        .rsplit_once("</think>")
        .map(|(_, rest)| rest)
        .unwrap_or(content);
    let trimmed = after_thinking.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Ok(value);
    }

    let start = trimmed
        .find('{')
        .ok_or_else(|| TeamderError::Internal("LLM response did not contain JSON".into()))?;
    let end = trimmed
        .rfind('}')
        .ok_or_else(|| TeamderError::Internal("LLM response did not contain JSON".into()))?;
    serde_json::from_str::<Value>(&trimmed[start..=end])
        .map_err(|e| TeamderError::Internal(format!("LLM JSON could not be parsed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_questions_from_json_block() {
        let content =
            "```json\n{\"questions\":[\"What did they own?\",\"How did they communicate?\"]}\n```";
        let questions = parse_questions(content).unwrap();
        assert_eq!(questions.len(), 2);
    }

    #[test]
    fn parses_summary_alias() {
        let summary = parse_summary("{\"review\":\"Clear and reliable.\"}").unwrap();
        assert_eq!(summary, "Clear and reliable.");
    }
}

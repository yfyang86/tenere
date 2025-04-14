use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::event::Event;
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::ChatGPTConfig;
use crate::llm::{LLMAnswer, LLMRole, LLM};

#[derive(Clone, Debug)]
pub struct ChatGPT {
    client: Client,
    openai_api_key: String,
    model: String,
    url: String,
    messages: Vec<HashMap<String, String>>,
}

impl ChatGPT {
    pub fn new(config: ChatGPTConfig) -> Self {
        let openai_api_key = config.openai_api_key.unwrap_or_else(|| {
            std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
                eprintln!(
                    "Missing OpenAI API key. Set OPENAI_API_KEY environment variable or config"
                );
                std::process::exit(1);
            })
        });

        Self {
            client: Client::new(),
            openai_api_key,
            model: config.model,
            url: config.url,
            messages: Vec::new(),
        }
    }
}

#[async_trait]
impl LLM for ChatGPT {
    fn clear(&mut self) {
        self.messages.clear();
    }

    fn append_chat_msg(&mut self, msg: String, role: LLMRole) {
        let mut conv = HashMap::new();
        conv.insert("role".to_string(), role.to_string());
        conv.insert("content".to_string(), msg);
        self.messages.push(conv);
    }

    async fn ask(
        &self,
        sender: UnboundedSender<Event>,
        terminate_response_signal: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.openai_api_key).parse()?,
        );

        let mut messages = vec![HashMap::from([
            ("role".to_string(), "system".to_string()),
            (
                "content".to_string(),
                "You are a helpful assistant.".to_string(),
            ),
        ])];
        messages.extend(self.messages.clone());

        let body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
        });

        let response = self
            .client
            .post(&self.url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        let mut buffer = Bytes::new();
        let mut line_buffer = String::new();

        sender.send(Event::LLMEvent(LLMAnswer::StartAnswer))?;

        while let Some(chunk_result) = stream.next().await {
            if terminate_response_signal.load(Ordering::Relaxed) {
                sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
                return Ok(());
            }

            let chunk = chunk_result?;
            buffer = Bytes::from([buffer.as_ref(), chunk.as_ref()].concat());

            // Process complete lines
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.split_to(newline_pos + 1);
                let line_str = String::from_utf8_lossy(&line[..line.len() - 1]); // Exclude \n

                process_sse_line(line_str.to_string(), &mut line_buffer, &sender)?;
            }
        }

        // Process remaining data
        if !buffer.is_empty() {
            let line_str = String::from_utf8_lossy(&buffer);
            process_sse_line(line_str.to_string(), &mut line_buffer, &sender)?;
        }

        sender.send(Event::LLMEvent(LLMAnswer::EndAnswer))?;
        Ok(())
    }
}

fn process_sse_line(
    line: String,
    line_buffer: &mut String,
    sender: &UnboundedSender<Event>,
) -> Result<(), Box<dyn std::error::Error>> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(());
    }

    // Skip id lines, we're only interested in data lines
    if line.starts_with("id:") {
        return Ok(());
    }

    // Process data lines
    if let Some(data) = line.strip_prefix("data:") {
        let data = data.trim();
        if data == "[DONE]" {
            return Ok(());
        }

        // Parse the JSON data
        match serde_json::from_str::<Value>(data) {
            Ok(json_value) => {
                if let Some(content) = json_value["choices"][0]["delta"]["content"].as_str() {
                    if !content.is_empty() {
                        sender.send(Event::LLMEvent(LLMAnswer::Answer(content.to_string())))?;
                    }
                }
            }
            Err(e) => {
                // Add to buffer for potential multi-line JSON
                line_buffer.push_str(data);

                // Try to parse the combined buffer
                match serde_json::from_str::<Value>(line_buffer) {
                    Ok(json_value) => {
                        if let Some(content) = json_value["choices"][0]["delta"]["content"].as_str() {
                            if !content.is_empty() {
                                sender.send(Event::LLMEvent(LLMAnswer::Answer(content.to_string())))?;
                            }
                        }
                        line_buffer.clear();
                    }
                    Err(_) => {
                        // If still can't parse, might be incomplete JSON
                        // Keep buffer for next chunk
                        if e.is_eof() {
                            // This is fine, wait for more data
                        } else {
                            // This is an actual parsing error
                            sender.send(Event::LLMEvent(LLMAnswer::Answer(
                                format!("[JSON PARSE ERROR: {}]", e).to_string(),
                            )))?;
                            line_buffer.clear();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
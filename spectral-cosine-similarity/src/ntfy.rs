use reqwest::blocking::Client;
use serde::Serialize;
use std::time::Duration;

const NTFY_BASE_URL: &str = "https://ntfy.sh";
const NTFY_TITLE: &str = "Spectral Benchmark Update";

#[derive(Clone, Debug)]
pub struct NtfyNotifier {
    topic: String,
    client: Client,
}

#[derive(Serialize)]
struct PublishPayload {
    topic: String,
    title: String,
    message: String,
    tags: Vec<String>,
    priority: u8,
    markdown: bool,
}

impl Default for NtfyNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl NtfyNotifier {
    pub fn new() -> Self {
        Self {
            topic: format!("bench-{:016x}", rand::random::<u64>()),
            client: Client::new(),
        }
    }

    pub fn subscription_url(&self) -> String {
        format!("{NTFY_BASE_URL}/{}", self.topic)
    }

    pub fn notify(&self, message: &str, tags: &[&str], priority: u8) {
        let payload = PublishPayload {
            topic: self.topic.clone(),
            title: NTFY_TITLE.to_string(),
            message: message.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            priority,
            markdown: true,
        };

        let result = self
            .client
            .post(NTFY_BASE_URL)
            .json(&payload)
            .send();
        if let Err(err) = result {
            eprintln!("[ntfy] WARNING: failed to publish notification: {err}");
        }
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut output = input.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    output
}

pub fn format_stage_completed(stage: &str, elapsed: Duration, key_count: Option<&str>) -> String {
    let key_line = match key_count {
        Some(value) => format!("- Key count: {value}"),
        None => "- Key count: n/a".to_string(),
    };
    format!(
        "### Section complete: {stage}\n- Status: success\n- Duration: {}\n{key_line}",
        format_duration(elapsed)
    )
}

pub fn format_pipeline_completed(elapsed: Duration) -> String {
    format!(
        "### Pipeline complete\n- Status: success\n- Duration: {}",
        format_duration(elapsed)
    )
}

pub fn format_pipeline_failed(stage_hint: Option<&str>, elapsed: Duration, error_summary: &str) -> String {
    let stage = stage_hint.unwrap_or("unknown");
    format!(
        "### Pipeline failed\n- Stage: {stage}\n- Elapsed: {}\n- Error: {}",
        format_duration(elapsed),
        truncate(error_summary, 240)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_is_hh_mm_ss() {
        let duration = Duration::from_secs(3 * 3600 + 4 * 60 + 5);
        assert_eq!(format_duration(duration), "03:04:05");
    }

    #[test]
    fn truncate_leaves_short_messages_and_cuts_long_messages() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghijk", 5), "abcde...");
    }
}

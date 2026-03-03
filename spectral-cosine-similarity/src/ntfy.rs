use reqwest::blocking::Client;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

const NTFY_BASE_URL: &str = "https://ntfy.sh";
const NTFY_TITLE: &str = "Spectral Benchmark Update";
const RETRY_DELAY_MS: u64 = 400;

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

impl NtfyNotifier {
    pub fn new_random_topic() -> Self {
        Self {
            topic: generate_uuid_v7(),
            client: Client::new(),
        }
    }

    pub fn subscription_url(&self) -> String {
        format!("{NTFY_BASE_URL}/{}", self.topic)
    }

    pub fn notify_stage_completed(&self, stage: &str, elapsed: Duration, key_count: Option<&str>) {
        let mut lines = vec![
            format!("### Section complete: {stage}"),
            "- Status: success".to_string(),
            format!("- Duration: {}", format_duration(elapsed)),
        ];
        lines.push(match key_count {
            Some(value) => format!("- Key count: {value}"),
            None => "- Key count: n/a".to_string(),
        });

        let payload = PublishPayload {
            topic: self.topic.clone(),
            title: NTFY_TITLE.to_string(),
            message: lines.join("\n"),
            tags: vec!["white_check_mark".to_string(), "test_tube".to_string()],
            priority: 3,
            markdown: true,
        };
        self.publish_best_effort(&payload);
    }

    pub fn notify_compute_step_completed(&self, step: &str, elapsed: Duration, rows_added: u64) {
        let payload = PublishPayload {
            topic: self.topic.clone(),
            title: NTFY_TITLE.to_string(),
            message: [
                format!("### Compute step complete: {step}"),
                "- Status: success".to_string(),
                format!("- Duration: {}", format_duration(elapsed)),
                format!("- Rows added: {rows_added}"),
            ]
            .join("\n"),
            tags: vec!["white_check_mark".to_string(), "gear".to_string()],
            priority: 3,
            markdown: true,
        };
        self.publish_best_effort(&payload);
    }

    pub fn notify_pipeline_completed(&self, elapsed: Duration) {
        let payload = PublishPayload {
            topic: self.topic.clone(),
            title: NTFY_TITLE.to_string(),
            message: [
                "### Pipeline complete".to_string(),
                "- Status: success".to_string(),
                format!("- Duration: {}", format_duration(elapsed)),
            ]
            .join("\n"),
            tags: vec!["bar_chart".to_string(), "tada".to_string()],
            priority: 3,
            markdown: true,
        };
        self.publish_best_effort(&payload);
    }

    pub fn notify_pipeline_failed(
        &self,
        stage_hint: Option<&str>,
        elapsed: Duration,
        error_summary: &str,
    ) {
        let stage = stage_hint.unwrap_or("unknown");
        let payload = PublishPayload {
            topic: self.topic.clone(),
            title: NTFY_TITLE.to_string(),
            message: [
                "### Pipeline failed".to_string(),
                format!("- Stage: {stage}"),
                format!("- Elapsed: {}", format_duration(elapsed)),
                format!("- Error: {}", truncate(error_summary, 240)),
            ]
            .join("\n"),
            tags: vec!["x".to_string(), "warning".to_string()],
            priority: 5,
            markdown: true,
        };
        self.publish_best_effort(&payload);
    }

    fn publish_best_effort(&self, payload: &PublishPayload) {
        if let Err(err) = send_with_retry(|| self.send_once(payload)) {
            eprintln!("[ntfy] WARNING: failed to publish notification: {err}");
        }
    }

    fn send_once(&self, payload: &PublishPayload) -> Result<(), String> {
        let response = self
            .client
            .post(NTFY_BASE_URL)
            .json(payload)
            .send()
            .map_err(|err| err.to_string())?;
        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let body = response
            .text()
            .unwrap_or_else(|_| "<failed to read response body>".to_string());
        Err(format!("HTTP {status}: {body}"))
    }
}

fn send_with_retry<F>(mut send_once: F) -> Result<(), String>
where
    F: FnMut() -> Result<(), String>,
{
    match send_once() {
        Ok(()) => Ok(()),
        Err(first_err) => {
            thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            send_once().map_err(|retry_err| {
                format!("first attempt failed ({first_err}); retry failed ({retry_err})")
            })
        }
    }
}

fn format_duration(duration: Duration) -> String {
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

fn generate_uuid_v7() -> String {
    let mut bytes = [0_u8; 16];
    fill_random_bytes(&mut bytes).unwrap_or_else(|_| fill_fallback_bytes(&mut bytes));

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u64, |duration| duration.as_millis() as u64);
    let ts = now_ms.to_be_bytes();
    bytes[0..6].copy_from_slice(&ts[2..8]);

    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-\
         {:02x}{:02x}-\
         {:02x}{:02x}-\
         {:02x}{:02x}-\
         {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn fill_random_bytes(bytes: &mut [u8]) -> std::io::Result<()> {
    let mut random = File::open("/dev/urandom")?;
    random.read_exact(bytes)?;
    Ok(())
}

fn fill_fallback_bytes(bytes: &mut [u8]) {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let mut state = now_nanos ^ ((std::process::id() as u128) << 64);
    for byte in bytes {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *byte = (state & 0xff) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn is_topic_safe(topic: &str) -> bool {
        !topic.is_empty()
            && topic
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    }

    #[test]
    fn random_topic_is_safe_and_uses_uuid_shape() {
        let notifier = NtfyNotifier::new_random_topic();
        let topic = notifier.subscription_url();
        let suffix = topic
            .strip_prefix("https://ntfy.sh/")
            .expect("missing ntfy base URL");
        assert!(is_topic_safe(suffix));
        assert_eq!(suffix.len(), 36);
        assert_eq!(&suffix[14..15], "7");
    }

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

    #[test]
    fn send_with_retry_retries_once_after_failure() {
        let calls = AtomicUsize::new(0);
        let result = send_with_retry(|| {
            let call_index = calls.fetch_add(1, Ordering::SeqCst);
            if call_index == 0 {
                Err("first failure".to_string())
            } else {
                Ok(())
            }
        });
        assert!(result.is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}

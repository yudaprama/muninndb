use futures::Stream;
use futures::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::errors::MuninnError;

/// A single Server-Sent Event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: String,
    pub data: String,
}

/// Parses an SSE stream from a `reqwest::Response` into a stream of `SseEvent`.
///
/// Each event is yielded when a blank line is encountered, matching the SSE spec.
pub fn parse_sse_stream(
    response: reqwest::Response,
) -> impl Stream<Item = Result<SseEvent, MuninnError>> {
    let (tx, rx) = mpsc::channel(64);

    tokio::spawn(async move {
        let byte_stream = response.bytes_stream();
        let mut reader = BufReader::new(tokio_util::io::StreamReader::new(
            byte_stream.map(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))),
        ));

        let mut event_type = String::from("message");
        let mut data_lines: Vec<String> = Vec::new();

        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        // Blank line = end of event
                        if !data_lines.is_empty() {
                            let data = data_lines.join("\n");
                            let event = SseEvent {
                                event_type: event_type.clone(),
                                data,
                            };
                            if tx.send(Ok(event)).await.is_err() {
                                break;
                            }
                            data_lines.clear();
                            event_type = "message".to_string();
                        }
                    } else if let Some(rest) = trimmed.strip_prefix("event:") {
                        event_type = rest.trim().to_string();
                    } else if let Some(rest) = trimmed.strip_prefix("data:") {
                        data_lines.push(rest.trim().to_string());
                    }
                    // Ignore comments (:) and other fields
                }
                Err(e) => {
                    let _ = tx.send(Err(MuninnError::ConnectionFailed(e.to_string()))).await;
                    break;
                }
            }
        }

        // Yield any remaining partial event
        if !data_lines.is_empty() {
            let data = data_lines.join("\n");
            let _ = tx
                .send(Ok(SseEvent {
                    event_type,
                    data,
                }))
                .await;
        }
    });

    ReceiverStream::new(rx)
}

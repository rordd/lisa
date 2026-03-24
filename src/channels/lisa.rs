//! LisaChannel — WebSocket transport channel (bridge pattern).
//!
//! The channel connects ZeroClaw's channel subsystem to browser/app clients
//! over a WebSocket connection served by the gateway at the `/app` endpoint.
//!
//! ## Architecture
//!
//! ```text
//! Browser ──WS──► gateway /app ──(push_message)──► LisaChannel.incoming_tx
//!                                                           │
//!                                              start_channels message bus
//!                                                           │
//!                                          process_channel_message()
//!                                                           │
//!                                             LisaChannel.send()
//!                                                           │
//!               Browser ◄──WS── out_tx (per-session) ◄─────┘
//! ```
//!
//! A module-level `OnceLock<Arc<LisaChannel>>` ensures that the gateway and
//! the channel subsystem share the same instance when running inside the same
//! daemon process.

use super::traits::{Channel, ChannelMessage, DataPart, SendMessage};
use async_trait::async_trait;
use axum::extract::ws::Message;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc;

/// Buffer for the incoming-message bridge (WS frames → channel bus).
const LISA_BRIDGE_BUFFER: usize = 256;

/// Outgoing WS frame sender handle for a single client connection.
type WsSender = mpsc::Sender<Message>;

/// Module-level singleton so gateway and start_channels share the same instance.
static GLOBAL: OnceLock<Arc<LisaChannel>> = OnceLock::new();

/// LisaChannel bridges gateway WebSocket connections to the ZeroClaw channel
/// subsystem.  It is always accessed through [`LisaChannel::global()`].
pub struct LisaChannel {
    /// Incoming ChannelMessages from WS clients, buffered for listen().
    incoming_tx: mpsc::Sender<ChannelMessage>,
    /// Held by listen() and taken out on the first call.
    incoming_rx: Mutex<Option<mpsc::Receiver<ChannelMessage>>>,
    /// Active connections: session_id → per-connection WS output channel.
    connections: Mutex<HashMap<String, WsSender>>,
}

impl LisaChannel {
    fn new() -> Self {
        let (incoming_tx, incoming_rx) = mpsc::channel(LISA_BRIDGE_BUFFER);
        Self {
            incoming_tx,
            incoming_rx: Mutex::new(Some(incoming_rx)),
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Return (or lazily create) the global LisaChannel singleton.
    pub fn global() -> Arc<LisaChannel> {
        Arc::clone(GLOBAL.get_or_init(|| Arc::new(LisaChannel::new())))
    }

    /// Push an incoming ChannelMessage from a WS client into the channel bus.
    ///
    /// Called by the gateway `/app` WebSocket handler for each user message.
    pub fn push_message(&self, msg: ChannelMessage) {
        if let Err(e) = self.incoming_tx.try_send(msg) {
            tracing::warn!("LisaChannel: message dropped (buffer full): {e}");
        }
    }

    /// Register a WS connection.  The `sender` receives all outgoing WS
    /// frames for this `session_id` until [`deregister`] is called.
    ///
    /// [`deregister`]: LisaChannel::deregister
    pub fn register(&self, session_id: String, sender: WsSender) {
        let mut conns = self.connections.lock();
        if let Some(old) = conns.insert(session_id.clone(), sender) {
            // Close the previous connection so it doesn't silently hang.
            tracing::info!(session_id, "LisaChannel: replaced existing connection");
            drop(old); // dropping UnboundedSender closes the channel
        }
    }

    /// Remove the WS sender for `session_id` (call on disconnect).
    pub fn deregister(&self, session_id: &str) {
        self.connections.lock().remove(session_id);
    }
}

#[async_trait]
impl Channel for LisaChannel {
    fn name(&self) -> &str {
        "lisa"
    }

    fn supports_a2ui(&self) -> bool {
        true
    }

    // Delta streaming deferred to Issue #77 (provider-level SSE streaming)

    async fn listen(&self, tx: mpsc::Sender<ChannelMessage>) -> anyhow::Result<()> {
        // The receiver may only be consumed once; take it out of the Option.
        let mut rx = self
            .incoming_rx
            .lock()
            .take()
            .ok_or_else(|| anyhow::anyhow!("LisaChannel: listen() called more than once"))?;

        tracing::info!(
            "Lisa channel active (WebSocket mode). \
            Clients connect to the gateway /app endpoint."
        );

        while let Some(msg) = rx.recv().await {
            if tx.send(msg).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
        let session_id = &message.recipient;
        let sender = self.connections.lock().get(session_id).cloned();

        let Some(sender) = sender else {
            tracing::debug!(session_id, "LisaChannel: no active connection for session");
            return Ok(());
        };

        // Send data parts first so the UI can render cards before the text arrives.
        if let Some(parts) = &message.data {
            for part in parts {
                match part {
                    DataPart::A2ui(messages) => {
                        let frame = serde_json::json!({
                            "type": "a2ui",
                            "messages": messages,
                        });
                        if sender.try_send(Message::Text(frame.to_string().into())).is_err() { tracing::warn!(session_id, "LisaChannel: outbound buffer full, frame dropped"); }
                    }
                    DataPart::A2web { url, id, title } => {
                        let frame = serde_json::json!({
                            "type": "a2web",
                            "url": url,
                            "id": id,
                            "title": title,
                        });
                        if sender.try_send(Message::Text(frame.to_string().into())).is_err() { tracing::warn!(session_id, "LisaChannel: outbound buffer full, frame dropped"); }
                    }
                }
            }
        }

        // Send done frame when there is text or data so the client can finalize rendering.
        let has_data = message.data.as_ref().is_some_and(|d| !d.is_empty());
        if !message.content.is_empty() || has_data {
            let mut done_frame = serde_json::json!({ "type": "done" });
            if !message.content.is_empty() {
                done_frame["full_response"] = serde_json::Value::String(message.content.clone());
            }
            if sender.try_send(Message::Text(done_frame.to_string().into())).is_err() { tracing::warn!(session_id, "LisaChannel: outbound buffer full, frame dropped"); }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lisa_channel_name_and_a2ui_flag() {
        let ch = LisaChannel::new();
        assert_eq!(ch.name(), "lisa");
        assert!(ch.supports_a2ui());
    }

    #[tokio::test]
    async fn listen_forwards_pushed_messages() {
        let ch = Arc::new(LisaChannel::new());
        let (tx, mut rx) = mpsc::channel(4);

        // Spawn listen() first so it runs concurrently with the push.
        let ch2 = Arc::clone(&ch);
        let handle = tokio::spawn(async move { ch2.listen(tx).await.unwrap_or_default() });

        ch.push_message(ChannelMessage {
            id: "1".into(),
            sender: "sess_abc".into(),
            reply_target: "sess_abc".into(),
            content: "hello from browser".into(),
            channel: "lisa".into(),
            timestamp: 0,
            thread_ts: None,
        });

        let msg = rx.recv().await.expect("should receive a message");
        assert_eq!(msg.content, "hello from browser");
        assert_eq!(msg.channel, "lisa");

        // listen() blocks waiting for more messages; abort the task to clean up.
        handle.abort();
    }

    #[tokio::test]
    async fn send_routes_text_to_registered_connection() {
        let ch = LisaChannel::new();
        let (out_tx, mut out_rx) = mpsc::channel::<Message>(256);

        ch.register("sess_xyz".into(), out_tx);

        ch.send(&SendMessage::new("hello", "sess_xyz"))
            .await
            .unwrap();

        let frame = out_rx.recv().await.expect("should receive WS frame");
        if let Message::Text(text) = frame {
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(v["type"], "done");
            assert_eq!(v["full_response"], "hello");
        } else {
            panic!("expected Text frame");
        }
    }

    #[tokio::test]
    async fn send_delivers_a2ui_parts_before_text() {
        let ch = LisaChannel::new();
        let (out_tx, mut out_rx) = mpsc::channel::<Message>(256);

        ch.register("sess_a2ui".into(), out_tx);

        let msg = SendMessage::new("here is the card", "sess_a2ui").with_data(vec![
            DataPart::A2ui(vec![serde_json::json!({"createSurface": {"surfaceId": "s1"}})]),
        ]);
        ch.send(&msg).await.unwrap();

        let first = out_rx.recv().await.expect("a2ui frame");
        let second = out_rx.recv().await.expect("done frame");

        if let Message::Text(t) = &first {
            let v: serde_json::Value = serde_json::from_str(t).unwrap();
            assert_eq!(v["type"], "a2ui");
        } else {
            panic!("expected Text frame for a2ui");
        }
        if let Message::Text(t) = &second {
            let v: serde_json::Value = serde_json::from_str(t).unwrap();
            assert_eq!(v["type"], "done");
        } else {
            panic!("expected Text frame for done");
        }
    }

    #[tokio::test]
    async fn send_to_missing_session_is_noop() {
        let ch = LisaChannel::new();
        assert!(ch
            .send(&SendMessage::new("hi", "nonexistent_session"))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn deregister_removes_connection() {
        let ch = LisaChannel::new();
        let (out_tx, _out_rx) = mpsc::channel::<Message>(256);
        ch.register("sess_drop".into(), out_tx);
        ch.deregister("sess_drop");
        // Sending after deregister is a no-op
        assert!(ch
            .send(&SendMessage::new("hi", "sess_drop"))
            .await
            .is_ok());
    }
}

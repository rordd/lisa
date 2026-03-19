//! Voice module — Realtime API voice sessions with Memory integration.
//!
//! This module bridges the `RealtimeProvider` (WebSocket streaming) with
//! the agent's `Memory` system, automatically storing conversation transcripts.

pub mod cli;
pub mod session;
pub mod web;

pub use session::VoiceSession;
#[allow(unused_imports)]
pub use web::run_voice_web;
pub use web::run_voice_web_with_agent;

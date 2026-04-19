// State Tauri pour le streaming cloud.

use parking_lot::Mutex;
use std::sync::Arc;

use crate::transcription::cloud::streaming::{StreamingHandle, StreamingRegistry};

pub struct StreamingRegistryState(pub Arc<StreamingRegistry>);

impl Default for StreamingRegistryState {
    fn default() -> Self {
        Self(Arc::new(StreamingRegistry::default()))
    }
}

/// Session streaming active pendant un enregistrement. None si pas de
/// streaming en cours (mode local / batch).
#[derive(Default)]
pub struct StreamingSessionState(pub Mutex<Option<StreamingHandle>>);

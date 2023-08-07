use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

use serde::{Deserialize, Serialize};

use usdpl_back::AsyncCallable;
use usdpl_back::core::serdes::Primitive;

use limits_core::json::DeveloperMessage;

use crate::MESSAGE_SEEN_ID_FILE;
use crate::utility::settings_dir;

#[derive(Serialize, Deserialize)]
pub struct ApiMessage {
    /// Message identifier
    pub id: Option<u64>,
    /// Message title
    pub title: String,
    /// Message content
    pub body: String,
    /// Link for further information
    pub url: Option<String>,
}

impl std::convert::From<DeveloperMessage> for ApiMessage {
    fn from(other: DeveloperMessage) -> Self {
        Self {
            id: Some(other.id),
            title: other.title,
            body: other.body,
            url: other.url,
        }
    }
}

fn get_dev_messages() -> Vec<ApiMessage> {
    crate::settings::get_dev_messages().drain(..).map(|msg| ApiMessage::from(msg)).collect()
}

pub struct MessageHandler {
    seen: Arc<AtomicU64>,
}

impl MessageHandler {
    pub fn new() -> Self {
        let last_seen_id = if let Ok(last_seen_id_bytes) = std::fs::read(settings_dir().join(MESSAGE_SEEN_ID_FILE)) {
            if last_seen_id_bytes.len() >= 8 /* bytes in u64 */ {
                u64::from_le_bytes([
                    last_seen_id_bytes[0],
                    last_seen_id_bytes[1],
                    last_seen_id_bytes[2],
                    last_seen_id_bytes[3],
                    last_seen_id_bytes[4],
                    last_seen_id_bytes[5],
                    last_seen_id_bytes[6],
                    last_seen_id_bytes[7],
                ])
            } else {
                u64::MAX
            }
        } else {
            u64::MIN
        };
        Self {
            seen: Arc::new(AtomicU64::new(last_seen_id)),
        }
    }

    pub fn to_callables(self) -> (AsyncMessageGetter, AsyncMessageDismisser) {
        (
            AsyncMessageGetter {
                seen: self.seen.clone(),
            },
            AsyncMessageDismisser {
                seen: self.seen.clone(),
            }
        )
    }
}

pub struct AsyncMessageGetter {
    seen: Arc<AtomicU64>,
}

impl AsyncMessageGetter {
    fn remove_before_id(id: u64, messages: impl Iterator<Item=ApiMessage>) -> impl Iterator<Item=ApiMessage> {
        messages.skip_while(move |msg| if let Some(msg_id) = msg.id { msg_id <= id } else { true })
    }
}

#[async_trait::async_trait]
impl AsyncCallable for AsyncMessageGetter {
    async fn call(&self, params: super::ApiParameterType) -> super::ApiParameterType {
        let since = if let Some(param0) = params.get(0) {
            if let Primitive::Empty = param0 {
                self.seen.load(Ordering::Relaxed)
            } else if let Primitive::U64(since) = param0 {
                *since
            } else {
                return vec!["get message invalid parameter 0".into()];
            }
        } else {
            self.seen.load(Ordering::Relaxed)
        };
        let mut messages = get_dev_messages();
        Self::remove_before_id(since, messages.drain(..))
            .filter_map(|msg| serde_json::to_string(&msg).ok().map(|x| Primitive::Json(x)))
            .collect()
    }
}

pub struct AsyncMessageDismisser {
    seen: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl AsyncCallable for AsyncMessageDismisser {
    async fn call(&self, params: super::ApiParameterType) -> super::ApiParameterType {
        let id = if let Some(param0) = params.get(0) {
            if let Primitive::Empty = param0 {
                None
            } else if let Primitive::F64(since) = param0 {
                Some(*since as u64)
            } else {
                return vec!["dismiss message invalid parameter 0".into()];
            }
        } else {
            None
        };
        if let Some(id) = id {
            self.seen.store(id, Ordering::Relaxed);
            let filename = settings_dir().join(MESSAGE_SEEN_ID_FILE);
            if let Err(e) = std::fs::write(&filename, id.to_le_bytes()) {
                log::error!("Failed to write seen id to {}: {}", filename.display(), e);
            }
        } else {
            // TODO clear non-dev messages in cache
        }
        vec![true.into()]
    }
}

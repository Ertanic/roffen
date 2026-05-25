use crate::utils::AsyncOption;
use dashmap::DashMap;
use fluent::{FluentResource, concurrent::FluentBundle};
use log::warn;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub type LangBundle = Arc<RwLock<FluentBundle<FluentResource>>>;

#[derive(Deserialize)]
pub struct LangMetaEntry {
    pub code: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct LangMeta {
    pub lang: Vec<LangMetaEntry>,
}

struct LangManagerInner {
    langs: Mutex<Vec<LangMetaEntry>>,
    bundles: DashMap<String, LangBundle>,
    current_lang: RwLock<String>,
    default_lang: RwLock<String>,
}

#[derive(Clone)]
pub struct LangManager(Arc<LangManagerInner>);

impl LangManager {
    pub fn new(current_lang: String, default_lang: String, bundles: DashMap<String, LangBundle>, langs: Vec<LangMetaEntry>) -> Self {
        Self(Arc::new(LangManagerInner {
            langs: Mutex::new(langs),
            bundles,
            current_lang: RwLock::new(current_lang),
            default_lang: RwLock::new(default_lang),
        }))
    }

    pub async fn get_message(&self, key: &str) -> String {
        self.try_get_message(key).await.expect("message not found")
    }

    pub async fn try_get_message(&self, key: &str) -> Option<String> {
        let bundle = self
            .0
            .bundles
            .get(&*self.0.current_lang.read().await)
            .async_or_else(|| async { self.0.bundles.get(&*self.0.default_lang.read().await) })
            .await;

        if let Some(bundle) = bundle {
            let bundle = bundle.read().await;

            let default_bundle = self
                .0
                .bundles
                .get(&*self.0.default_lang.read().await)
                .expect("no default lang bundle found");
            let default_bundle = default_bundle.read().await;

            let mut errors = vec![];
            let message = match bundle.get_message(key) {
                None => {
                    warn!(
                        "message not found into {} bundle: '{}', search into default bundle",
                        self.0.current_lang.read().await,
                        key
                    );

                    match default_bundle.get_message(key) {
                        Some(message) => message,
                        None => {
                            warn!("message not found into default bundle: '{}'", key);
                            return None;
                        }
                    }
                }
                Some(message) => message,
            };

            let val = message.value()?;
            let message = bundle.format_pattern(val, None, &mut errors);

            if !errors.is_empty() {
                warn!("in message {key} errors: {errors:?}");
            }

            Some(message.to_string())
        }
        else {
            None
        }
    }

    pub async fn replace_meta(&self, meta: LangMeta) {
        *self.0.langs.lock().await = meta.lang;
    }

    pub async fn set_current(&self, lang: String) {
        *self.0.current_lang.write().await = lang;
    }

    pub async fn set_default(&self, lang: String) {
        *self.0.default_lang.write().await = lang;
    }

    pub async fn replace_lang(&self, lang: String, bundle: LangBundle) {
        self.0.bundles.insert(lang, bundle);
    }
}

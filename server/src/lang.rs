use dashmap::DashMap;
use fluent::{FluentResource, concurrent::FluentBundle};
use log::warn;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type LangBundle = Arc<RwLock<FluentBundle<FluentResource>>>;

#[derive(Deserialize)]
pub struct LangMetaEntry {
    pub code: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct ServerLangConfig {
    pub current: String,
    pub default: String,
}

#[derive(Deserialize)]
pub struct LangMeta {
    pub server: ServerLangConfig,
    pub lang: Vec<LangMetaEntry>,
}

struct LangManagerInner {
    langs: Vec<LangMetaEntry>,
    bundles: DashMap<String, LangBundle>,
    current_lang: String,
    default_lang: String,
}

#[derive(Clone)]
pub struct LangManager(Arc<LangManagerInner>);

impl LangManager {
    pub fn new(current_lang: String, default_lang: String, bundles: DashMap<String, LangBundle>, langs: Vec<LangMetaEntry>) -> Self {
        Self(Arc::new(LangManagerInner {
            langs,
            bundles,
            current_lang,
            default_lang,
        }))
    }

    pub async fn get_message(&self, key: &str) -> String {
        self.try_get_message(key).await.expect("message not found")
    }

    pub async fn try_get_message(&self, key: &str) -> Option<String> {
        let bundle = self
            .0
            .bundles
            .get(&self.0.current_lang)
            .or_else(|| self.0.bundles.get(&self.0.default_lang));

        if let Some(bundle) = bundle {
            let bundle = bundle.read().await;

            let default_bundle = self.0.bundles.get(&self.0.default_lang).expect("no default lang bundle found");
            let default_bundle = default_bundle.read().await;

            let mut errors = vec![];
            let message = match bundle.get_message(key) {
                None => {
                    warn!("message not found into {} bundle: '{}'", self.0.current_lang, key);

                    match default_bundle.get_message(key) {
                        Some(message) => message,
                        None => return None,
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
}

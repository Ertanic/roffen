#[cfg(debug_assertions)]
pub const DEFAULT_ADDR: &str = "127.0.0.1";
#[cfg(not(debug_assertions))]
pub const DEFAULT_ADDR: &str = "0.0.0.0";

#[cfg(debug_assertions)]
pub const DEFAULT_HTTP_PORT: u16 = 8080;
#[cfg(not(debug_assertions))]
pub const DEFAULT_HTTP_PORT: u16 = 80;
#[cfg(debug_assertions)]
pub const DEFAULT_HTTPS_PORT: u16 = 8443;
#[cfg(not(debug_assertions))]
pub const DEFAULT_HTTPS_PORT: u16 = 443;

pub const LOGS_FILENAME: &str = "server.log";
pub const CONTENT_FOLDER: &str = "content";
pub const CERTS_FOLDER: &str = "certs";
pub const PUBLIC_FOLDER: &str = "public";
pub const PAGES_FOLDER: &str = "pages";
pub const INDEX_FILENAME: &str = "index.html";
pub const CONFIG_FILENAME: &str = "config.toml";
pub const AUTH_EXP: usize = 60 * 60 * 24; // day
pub const AUTH_COOKIE_NAME: &str = "auth";
pub const POSTS_FOLDER: &str = "posts";
pub const COMPS_FOLDER: &str = "components";
pub const DEFAULT_LANG_CODE: &str = "en-US";
pub const LANG_FOLDER: &str = "lang";
pub const LANG_META_FILE: &str = "meta.toml";
pub const COMPS_META_FILE: &str = "meta.ron";
pub const COMPS_JS_FILE: &str = "index.js";
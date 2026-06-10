//! Localized string lookup with 3-tier fallback.
//!
//! Resolution order (RFC 003 §2.3):
//! 1. Language + Region (`ar-SY`)
//! 2. Language only (`ar`)
//! 3. Fallback (`en` or pictogram-only mode)

use std::collections::HashMap;

use crate::locale::LocaleTag;

/// A flat map of string keys to translated values for one locale.
pub type StringMap = HashMap<String, String>;

/// A bundle of locale string maps, indexed by locale tag string.
pub struct I18nBundle {
    maps: HashMap<String, StringMap>,
    /// The fallback locale used when a key is missing in the requested locale.
    fallback: String,
}

impl I18nBundle {
    /// Create a new bundle. `fallback` should be `"en"` or a minimal locale.
    pub fn new(fallback: impl Into<String>) -> Self {
        Self {
            maps: HashMap::new(),
            fallback: fallback.into(),
        }
    }

    /// Register a string map for `locale_tag`.
    pub fn add_locale(&mut self, locale_tag: impl Into<String>, map: StringMap) {
        self.maps.insert(locale_tag.into(), map);
    }

    /// Look up `key` for `locale` using 3-tier fallback.
    ///
    /// Returns `None` only if the key is absent from every available tier,
    /// including the fallback locale.
    pub fn get(&self, locale: &LocaleTag, key: &str) -> Option<&str> {
        // Tier 1: exact tag (e.g. "ar-SY")
        if let Some(v) = self.maps.get(&locale.0).and_then(|m| m.get(key)) {
            return Some(v.as_str());
        }
        // Tier 2: language only (e.g. "ar")
        let lang = locale.language();
        if lang != locale.0 {
            if let Some(v) = self.maps.get(lang).and_then(|m| m.get(key)) {
                return Some(v.as_str());
            }
        }
        // Tier 3: fallback locale
        if lang != self.fallback {
            if let Some(v) = self.maps.get(&self.fallback).and_then(|m| m.get(key)) {
                return Some(v.as_str());
            }
        }
        None
    }

    /// Look up a key, returning `key` itself when no translation is found.
    ///
    /// This ensures the UI always displays something rather than a blank.
    pub fn t<'a>(&'a self, locale: &LocaleTag, key: &'a str) -> &'a str {
        self.get(locale, key).unwrap_or(key)
    }
}

//! BCP 47 locale model.

use serde::{Deserialize, Serialize};

use crate::direction::TextDirection;

/// A locale tag conforming to BCP 47 (e.g. `"ar"`, `"en-US"`, `"sw-KE"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocaleTag(pub String);

impl LocaleTag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    /// Returns the primary language subtag (the part before the first `-`).
    pub fn language(&self) -> &str {
        self.0.split('-').next().unwrap_or(&self.0)
    }

    /// Returns the region subtag if present (e.g. `"US"` from `"en-US"`).
    pub fn region(&self) -> Option<&str> {
        let mut parts = self.0.splitn(3, '-');
        parts.next(); // language
        let second = parts.next()?;
        // A region subtag is 2 letters (ISO 3166-1) or 3 digits.
        if second.len() == 2 && second.chars().all(|c| c.is_ascii_alphabetic()) {
            Some(second)
        } else {
            None
        }
    }

    /// Infer the text direction for this locale.
    pub fn direction(&self) -> TextDirection {
        TextDirection::for_language(self.language())
    }
}

impl std::fmt::Display for LocaleTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A locale pack: metadata for one installed language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalePack {
    pub locale_id: String,
    pub bcp47_tag: LocaleTag,
    /// Display name in the locale's own language (e.g. `"العربية"` for Arabic).
    pub language_name_local: Option<String>,
    pub direction: TextDirection,
    pub installed_at: i64,
    pub status: LocalePackStatus,
}

/// Status of an installed locale pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalePackStatus {
    Active,
    Disabled,
}

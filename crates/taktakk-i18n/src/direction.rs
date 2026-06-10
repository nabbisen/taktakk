//! Text direction (LTR / RTL) model.

use serde::{Deserialize, Serialize};

/// Writing direction for a locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    /// Left-to-right (Latin, Cyrillic, …).
    Ltr,
    /// Right-to-left (Arabic, Hebrew, Thaana, …).
    Rtl,
}

impl TextDirection {
    /// Infer the direction for a BCP 47 primary language subtag.
    ///
    /// Covers the RTL languages most relevant to taktakk's target regions.
    pub fn for_language(lang: &str) -> Self {
        match lang {
            "ar"   // Arabic
            | "he" | "iw" // Hebrew (both legacy and current code)
            | "fa" | "per" // Persian / Farsi
            | "ur"  // Urdu
            | "ps"  // Pashto
            | "sd"  // Sindhi
            | "ku"  // Kurdish (in Arabic script)
            | "ug"  // Uyghur
            | "yi"  // Yiddish
            | "dv"  // Divehi / Maldivian
            | "nqo" // N'Ko
            | "tfng" // Tifinagh (some variants)
            => Self::Rtl,
            _ => Self::Ltr,
        }
    }

    /// Returns the HTML `dir` attribute value.
    pub fn html_attr(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

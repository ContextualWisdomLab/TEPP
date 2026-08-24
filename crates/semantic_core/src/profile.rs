//! Language profiles are metadata, not unit identity.

use crate::error::SemanticError;

/// A language profile that may select tailoring without becoming identity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LanguageProfile {
    /// No language metadata; span bounds stay the supplied exact coordinates.
    Unresolved,
    /// A canonical primary ISO 639 subtag with optional ISO 3166-1 region.
    Tagged {
        /// Lowercase `language` or `language-region` label.
        tag: String,
    },
}

impl LanguageProfile {
    /// Return the unresolved profile.
    #[must_use]
    pub const fn unresolved() -> Self {
        Self::Unresolved
    }

    /// Parse a primary language subtag with an optional region.
    ///
    /// Accepts a syntactically valid two- or three-letter primary language subtag,
    /// followed by a hyphen and either an ISO 3166-1 alpha-2 region or a
    /// three-digit UN M.49 numeric region (Phillips & Davis, 2009, section
    /// 2.2.4; for example `es-419`). The stored form is lowercase. Empty tags
    /// and any other shape fail closed. The tag never becomes a
    /// [`crate::SemanticIdentity`].
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyLanguageTag`] or
    /// [`SemanticError::InvalidLanguageTag`].
    pub fn parse_bcp47(tag: &str) -> Result<Self, SemanticError> {
        if tag.is_empty() {
            return Err(SemanticError::EmptyLanguageTag);
        }
        let canonical = tag.to_ascii_lowercase();
        if !is_primary_language_tag(&canonical) {
            return Err(SemanticError::InvalidLanguageTag);
        }
        Ok(Self::Tagged { tag: canonical })
    }

    /// Return whether this profile is unresolved.
    #[must_use]
    pub const fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved)
    }

    /// Return the stable profile label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Unresolved => "unresolved",
            Self::Tagged { tag } => tag,
        }
    }
}

/// Validate a primary language tag per RFC 5646 sections 2.2.1 and 2.2.4.
///
/// A region may be an ISO 3166-1 alpha-2 code or a UN M.49 three-digit
/// numeric subtag; both are accepted here so regional variants such as
/// Latin-American Spanish resolve instead of failing closed.
fn is_primary_language_tag(tag: &str) -> bool {
    match tag.split_once('-') {
        None => is_letter_run(tag, 2, 3),
        Some((language, region)) => {
            is_letter_run(language, 2, 3) && is_registered_region_subtag(region)
        }
    }
}

fn is_letter_run(value: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_lowercase())
}

// IANA Language Subtag Registry, File-Date 2026-08-08. Private-use region
// ranges and private-use records are intentionally excluded because this
// profile accepts only reproducible registered region metadata.
const REGISTERED_ALPHA2_REGION_SUBTAGS: &str = "ac ad ae af ag ai al am an ao aq ar as at au aw ax az ba bb bd be bf bg bh bi bj bl bm bn bo bq br bs bt bu bv bw by bz ca cc cd cf cg ch ci ck cl cm cn co cp cq cr cs cu cv cw cx cy cz dd de dg dj dk dm do dz ea ec ee eg eh er es et eu ez fi fj fk fm fo fr fx ga gb gd ge gf gg gh gi gl gm gn gp gq gr gs gt gu gw gy hk hm hn hr ht hu ic id ie il im in io iq ir is it je jm jo jp ke kg kh ki km kn kp kr kw ky kz la lb lc li lk lr ls lt lu lv ly ma mc md me mf mg mh mk ml mm mn mo mp mq mr ms mt mu mv mw mx my mz na nc ne nf ng ni nl no np nr nt nu nz om pa pe pf pg ph pk pl pm pn pr ps pt pw py qa re ro rs ru rw sa sb sc sd se sg sh si sj sk sl sm sn so sr ss st su sv sx sy sz ta tc td tf tg th tj tk tl tm tn to tp tr tt tv tw tz ua ug um un us uy uz va vc ve vg vi vn vu wf ws yd ye yt yu za zm zr zw";

const REGISTERED_NUMERIC_REGION_SUBTAGS: &[&str] = &[
    "001", "002", "003", "005", "009", "011", "013", "014", "015", "017", "018", "019", "021",
    "029", "030", "034", "035", "039", "053", "054", "057", "061", "142", "143", "145", "150",
    "151", "154", "155", "202", "419",
];

fn is_registered_region_subtag(region: &str) -> bool {
    if region.len() == 2 {
        return REGISTERED_ALPHA2_REGION_SUBTAGS
            .split_ascii_whitespace()
            .any(|candidate| candidate == region);
    }
    region.len() == 3
        && region.bytes().all(|byte| byte.is_ascii_digit())
        && REGISTERED_NUMERIC_REGION_SUBTAGS.contains(&region)
}

#[cfg(test)]
mod tests {
    use super::LanguageProfile;
    use crate::error::SemanticError;

    #[test]
    fn parse_accepts_primary_and_region_and_rejects_noise() {
        let korean = LanguageProfile::parse_bcp47("KO").expect("ko");
        assert_eq!(korean.as_str(), "ko");
        assert!(!korean.is_unresolved());
        let tagged = LanguageProfile::parse_bcp47("en-US").expect("en-us");
        assert_eq!(tagged.as_str(), "en-us");
        let numeric = LanguageProfile::parse_bcp47("es-419").expect("es-419");
        assert_eq!(numeric.as_str(), "es-419");
        assert_eq!(
            LanguageProfile::parse_bcp47("ES-419")
                .expect("upper")
                .as_str(),
            "es-419"
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-GB")
                .expect("registered alpha-2 region")
                .as_str(),
            "en-gb"
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("yue").expect("yue").as_str(),
            "yue"
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("").unwrap_err(),
            SemanticError::EmptyLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("english").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-US-x-private").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("e").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en_us").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("e1").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-u1").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("en-u").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("e-us").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("es-41a").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("es-41").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        assert_eq!(
            LanguageProfile::parse_bcp47("es-4199").unwrap_err(),
            SemanticError::InvalidLanguageTag
        );
        for tag in ["en-aa", "en-XX", "en-QM", "en-abc", "en-999"] {
            assert_eq!(
                LanguageProfile::parse_bcp47(tag),
                Err(SemanticError::InvalidLanguageTag),
                "private or unknown region must fail closed: {tag}"
            );
        }
        let unresolved = LanguageProfile::unresolved();
        assert!(unresolved.is_unresolved());
        assert_eq!(unresolved.as_str(), "unresolved");
    }
}

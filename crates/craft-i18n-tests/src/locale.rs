//! Locale directory names and how many plural forms a locale's `@plural` rows list.

use icu_locale_core::Locale;
use icu_plurals::PluralRules;
use unic_langid::LanguageIdentifier;

/// Ok when `name` is a BCP 47 tag written in lowercase (`pt-br`, `zh-hans`).
///
/// # Errors
///
/// Returns an error when `name` is not a valid BCP 47 tag or is not written in lowercase.
pub fn check_locale_name(name: &str) -> Result<(), String> {
    let id: LanguageIdentifier = name
        .parse()
        .map_err(|e| format!("{name:?} is not a BCP 47 locale: {e}"))?;
    let lower = id.to_string().to_lowercase();
    if lower == name {
        Ok(())
    } else {
        Err(format!("{name:?}: write it as {lower:?}"))
    }
}

/// How many plural forms `locale`'s `@plural` rows list: the CLDR cardinal categories whole
/// numbers 0..=1000 reach, counted as craft-libs' `craft-i18n` does (`plural_rules` there), on the
/// same `icu_plurals` version and CLDR data (48.2.1). Categories only fractions or huge numbers
/// reach (French `many`) don't count. The rules are looked up by language alone: CLDR's parent of
/// `sr-Latn` and `zh-Hant` is the root, whose only category is `other`. CLDR's `pt` is Brazilian,
/// so Portuguese other than Brazil's takes `pt-PT`'s rules. A language CLDR has no rules for gets
/// the root's one form, as in the crate; `tests/runtime_agreement.rs` fails on such a locale,
/// since the crate then shows its plurals in English.
///
/// # Errors
///
/// Returns an error when `locale` is not a valid BCP 47 tag.
pub fn plural_forms(locale: &str) -> Result<usize, String> {
    let id = Locale::try_from_str(locale)
        .map_err(|e| format!("{locale:?} is not a BCP 47 locale: {e}"))?
        .id;
    let language = id.language.as_str();
    let brazil = id.region.is_some_and(|r| r.as_str() == "BR");
    let rules_for = if language == "pt" && !brazil {
        "pt-PT"
    } else {
        language
    };
    let rules_locale = Locale::try_from_str(rules_for)
        .map_err(|e| format!("{locale:?} is not a BCP 47 locale: {e}"))?;
    let rules = PluralRules::try_new_cardinal((&rules_locale).into())
        .map_err(|e| format!("{locale:?}: no plural rules: {e}"))?;
    let mut seen = Vec::new();
    for n in 0..=1000_u32 {
        let category = rules.category_for(n);
        if !seen.contains(&category) {
            seen.push(category);
        }
    }
    Ok(seen.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_names() {
        for ok in ["en", "fr", "he", "pt-br", "zh-hans", "zh-hant", "sr-latn"] {
            assert!(check_locale_name(ok).is_ok(), "{ok}");
        }
        for bad in ["pt_br", "pt-BR", "zh-Hans", "PT", "", "not a locale"] {
            assert!(check_locale_name(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn plural_form_counts() {
        for (locale, n) in [
            ("en", 2),
            ("fr", 2),
            ("he", 3),
            ("ja", 1),
            ("vi", 1),
            ("ru", 3),
            ("pl", 3),
            ("cs", 3),
            ("sl", 4),
            ("ar", 6),
            ("pt", 2),
            ("pt-br", 2),
            ("tr", 2),
            ("sr-latn", 3),
            ("zh-hant", 1),
            ("pt-pt", 2),
        ] {
            assert_eq!(plural_forms(locale), Ok(n), "{locale}");
        }
    }
}

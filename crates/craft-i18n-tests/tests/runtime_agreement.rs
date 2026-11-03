//! The checker and craft-libs' `craft-i18n` runtime crate agree on how many plural forms each
//! locale in `locales/` takes. They disagreeing means a catalog CI passes has `@plural` rows the
//! apps skip, or the reverse.

use craft_i18n::I18n;
use craft_i18n_tests::locale::plural_forms;

/// The runtime crate's problems with one `@plural` row of `forms` forms for `locale`.
fn runtime_problems(locale: &str, forms: usize) -> Vec<String> {
    let translation = vec!["{n} x"; forms].join("|");
    let catalog = format!("@plural\t{{n}} a|{{n}} b\t{translation}\n");
    let embedded: &'static [(&'static str, &'static str, &'static str)] = Box::leak(Box::new([(
        &*Box::leak(locale.to_owned().into_boxed_str()),
        &*Box::leak(catalog.into_boxed_str()),
        "",
    )]));
    let (i18n, mut problems) = I18n::new(embedded, &[]);
    problems.extend(i18n.check_all());
    problems
}

#[test]
fn plural_form_counts_match_the_runtime_crate() {
    let dir = craft_i18n_tests::repo_root().join("locales");
    let mut locales: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    locales.sort();
    assert!(locales.len() > 30, "{locales:?}");
    let mut disagree = Vec::new();
    for locale in &locales {
        let n = plural_forms(locale).unwrap();
        // The crate takes exactly `n` forms: `n` passes, one fewer and one more don't.
        let ok = runtime_problems(locale, n);
        let fewer = n > 1 && runtime_problems(locale, n - 1).is_empty();
        let more = runtime_problems(locale, n + 1).is_empty();
        if !ok.is_empty() || fewer || more {
            disagree.push(format!("{locale}: checker {n}, runtime {ok:?}"));
        }
    }
    assert_eq!(disagree, Vec::<String>::new());
}

/// The comparison itself catches a wrong count.
#[test]
fn a_wrong_count_is_caught() {
    assert!(runtime_problems("fr", 2).is_empty());
    assert!(!runtime_problems("fr", 3).is_empty());
    assert!(!runtime_problems("ru", 2).is_empty());
    assert!(
        !runtime_problems("tlh", 1).is_empty(),
        "no CLDR plural rules"
    );
}

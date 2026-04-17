use std::sync::{Arc, OnceLock};

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::{LanguageIdentifier, langid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum AppLanguagePreference {
    Auto,
    English,
    SimplifiedChinese,
}

impl AppLanguagePreference {
    #[allow(dead_code)]
    pub(crate) const fn as_value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::English => "en-US",
            Self::SimplifiedChinese => "zh-Hans",
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_value(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "en-US" => Some(Self::English),
            "zh-Hans" => Some(Self::SimplifiedChinese),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SupportedLanguage {
    English,
    SimplifiedChinese,
}

impl SupportedLanguage {
    fn langid(self) -> LanguageIdentifier {
        match self {
            Self::English => langid!("en-US"),
            Self::SimplifiedChinese => langid!("zh-Hans"),
        }
    }
}

const EN_US_FTL: &str = include_str!("../../../assets/i18n/en-US.ftl");
const ZH_HANS_FTL: &str = include_str!("../../../assets/i18n/zh-Hans.ftl");

static EN_US_RESOURCE: OnceLock<Arc<FluentResource>> = OnceLock::new();
static ZH_HANS_RESOURCE: OnceLock<Arc<FluentResource>> = OnceLock::new();

pub(crate) fn detect_device_locale_tag() -> String {
    sys_locale::get_locale()
        .filter(|locale| !locale.trim().is_empty())
        .unwrap_or_else(|| "en-US".to_string())
}

pub(crate) fn resolve_language(
    preference: AppLanguagePreference,
    device_locale_tag: &str,
) -> SupportedLanguage {
    match preference {
        AppLanguagePreference::Auto => negotiate_device_language(device_locale_tag),
        AppLanguagePreference::English => SupportedLanguage::English,
        AppLanguagePreference::SimplifiedChinese => SupportedLanguage::SimplifiedChinese,
    }
}

fn negotiate_device_language(device_locale_tag: &str) -> SupportedLanguage {
    let normalized = device_locale_tag.replace('_', "-");
    if let Ok(locale) = normalized.parse::<LanguageIdentifier>()
        && locale.language.as_str() == "zh"
    {
        return SupportedLanguage::SimplifiedChinese;
    }
    if normalized.to_ascii_lowercase().starts_with("zh") {
        SupportedLanguage::SimplifiedChinese
    } else {
        SupportedLanguage::English
    }
}

#[allow(dead_code)]
pub(crate) fn language_name(preference_language: SupportedLanguage) -> String {
    match preference_language {
        SupportedLanguage::English => text(SupportedLanguage::English, "settings-language-english"),
        SupportedLanguage::SimplifiedChinese => text(
            SupportedLanguage::SimplifiedChinese,
            "settings-language-zh-hans",
        ),
    }
}

pub(crate) fn text(language: SupportedLanguage, key: &str) -> String {
    text_with_args(language, key, &[])
}

pub(crate) fn text_with_args(
    language: SupportedLanguage,
    key: &str,
    args: &[(&str, String)],
) -> String {
    let mut bundle = FluentBundle::new(vec![language.langid()]);
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource(language))
        .expect("valid fluent resource");

    let Some(message) = bundle.get_message(key) else {
        return key.to_string();
    };
    let Some(pattern) = message.value() else {
        return key.to_string();
    };

    let mut fluent_args = FluentArgs::new();
    for (name, value) in args {
        fluent_args.set(*name, value.clone());
    }

    let mut errors = Vec::new();
    bundle
        .format_pattern(
            pattern,
            (!args.is_empty()).then_some(&fluent_args),
            &mut errors,
        )
        .into_owned()
}

fn resource(language: SupportedLanguage) -> Arc<FluentResource> {
    match language {
        SupportedLanguage::English => EN_US_RESOURCE
            .get_or_init(|| parse_resource(EN_US_FTL))
            .clone(),
        SupportedLanguage::SimplifiedChinese => ZH_HANS_RESOURCE
            .get_or_init(|| parse_resource(ZH_HANS_FTL))
            .clone(),
    }
}

fn parse_resource(source: &str) -> Arc<FluentResource> {
    Arc::new(FluentResource::try_new(source.to_string()).expect("valid fluent resource"))
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WallpaperTemplate {
    Focus,
    Kanban,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguagePreference {
    System,
    Tr,
    En,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub template: WallpaperTemplate,
    pub opacity: u8,
    pub edit_mode: bool,
    pub monitor_id: Option<String>,
    pub auto_calm_minutes: Option<u16>,
    pub theme: ThemePreference,
    pub language: LanguagePreference,
}

impl AppSettings {
    pub fn validate(self) -> Result<Self, String> {
        if !(40..=100).contains(&self.opacity) {
            return Err("Saydamlık değeri 40 ile 100 arasında olmalıdır.".into());
        }
        if self
            .auto_calm_minutes
            .is_some_and(|minutes| !(1..=120).contains(&minutes))
        {
            return Err("Otomatik sakin mod süresi 1 ile 120 dakika arasında olmalıdır.".into());
        }
        Ok(self)
    }
}

impl WallpaperTemplate {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::Kanban => "kanban",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "focus" => Ok(Self::Focus),
            "kanban" => Ok(Self::Kanban),
            other => Err(format!("Bilinmeyen wallpaper şablonu: {other}")),
        }
    }
}

impl ThemePreference {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "system" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            other => Err(format!("Bilinmeyen tema tercihi: {other}")),
        }
    }
}

impl LanguagePreference {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Tr => "tr",
            Self::En => "en",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "system" => Ok(Self::System),
            "tr" => Ok(Self::Tr),
            "en" => Ok(Self::En),
            other => Err(format!("Bilinmeyen dil tercihi: {other}")),
        }
    }
}

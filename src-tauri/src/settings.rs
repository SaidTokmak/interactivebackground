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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundSource {
    Preset,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundPreset {
    FoldedHorizon,
    Midnight,
    Graphite,
    Ember,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BackgroundFit {
    Cover,
    Contain,
    Stretch,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundSettings {
    pub monitor_id: Option<String>,
    pub source: BackgroundSource,
    pub preset: BackgroundPreset,
    pub custom_path: Option<String>,
    pub fit: BackgroundFit,
    pub overlay: u8,
    pub blur: u8,
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

impl BackgroundSettings {
    pub fn defaults_for(monitor_id: Option<String>) -> Self {
        Self {
            monitor_id,
            source: BackgroundSource::Preset,
            preset: BackgroundPreset::FoldedHorizon,
            custom_path: None,
            fit: BackgroundFit::Cover,
            overlay: 16,
            blur: 0,
        }
    }

    pub fn validate(mut self) -> Result<Self, String> {
        if self.overlay > 70 {
            return Err("Arka plan karartması 0 ile 70 arasında olmalıdır.".into());
        }
        if self.blur > 24 {
            return Err("Arka plan bulanıklığı 0 ile 24 arasında olmalıdır.".into());
        }
        if self.source == BackgroundSource::Custom
            && self.custom_path.as_deref().is_none_or(str::is_empty)
        {
            return Err("Özel arka plan dosyası bulunamadı.".into());
        }
        if self.source == BackgroundSource::Preset {
            self.custom_path = None;
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

impl BackgroundSource {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Preset => "preset",
            Self::Custom => "custom",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "preset" => Ok(Self::Preset),
            "custom" => Ok(Self::Custom),
            other => Err(format!("Bilinmeyen arka plan kaynağı: {other}")),
        }
    }
}

impl BackgroundPreset {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::FoldedHorizon => "folded_horizon",
            Self::Midnight => "midnight",
            Self::Graphite => "graphite",
            Self::Ember => "ember",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "folded_horizon" => Ok(Self::FoldedHorizon),
            "midnight" => Ok(Self::Midnight),
            "graphite" => Ok(Self::Graphite),
            "ember" => Ok(Self::Ember),
            other => Err(format!("Bilinmeyen arka plan teması: {other}")),
        }
    }
}

impl BackgroundFit {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Cover => "cover",
            Self::Contain => "contain",
            Self::Stretch => "stretch",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "cover" => Ok(Self::Cover),
            "contain" => Ok(Self::Contain),
            "stretch" => Ok(Self::Stretch),
            other => Err(format!("Bilinmeyen arka plan ölçekleme biçimi: {other}")),
        }
    }
}

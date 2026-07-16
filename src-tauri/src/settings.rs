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
    Porcelain,
    Arctic,
    Linen,
    MorningMist,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetLayout {
    pub monitor_id: Option<String>,
    pub template: WallpaperTemplate,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub locked: bool,
    pub snap_to_grid: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WidgetKind {
    Focus,
    Kanban,
    Pomodoro,
    Clock,
    Date,
    DailyPoem,
    DailyVerse,
    DailyHadith,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WidgetPackageSource {
    Core,
    BundledStore,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetPackage {
    pub kind: WidgetKind,
    pub source: WidgetPackageSource,
    pub version: String,
    pub installed: bool,
    pub minimum_width: f64,
    pub minimum_height: f64,
    pub permissions: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClockStyle {
    Digital,
    Analog,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClockHourFormat {
    System,
    Hour12,
    Hour24,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClockWidgetSettings {
    pub version: u8,
    pub style: ClockStyle,
    pub hour_format: ClockHourFormat,
    pub time_zone: Option<String>,
    pub show_seconds: bool,
    pub show_date: bool,
    pub show_weekday: bool,
}

impl Default for ClockWidgetSettings {
    fn default() -> Self {
        Self {
            version: 1,
            style: ClockStyle::Digital,
            hour_format: ClockHourFormat::System,
            time_zone: None,
            show_seconds: true,
            show_date: true,
            show_weekday: true,
        }
    }
}

impl ClockWidgetSettings {
    pub fn validate(mut self) -> Result<Self, String> {
        if self.version != 1 {
            return Err("Desteklenmeyen saat ayarı sürümü.".into());
        }
        self.time_zone = self
            .time_zone
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if self.time_zone.as_ref().is_some_and(|zone| {
            zone.len() > 64
                || !zone.chars().all(|character| {
                    character.is_ascii_alphanumeric() || "/_+-".contains(character)
                })
        }) {
            return Err("Saat dilimi geçerli bir IANA tanımlayıcısı olmalıdır.".into());
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StarterLayout {
    Focus,
    Planning,
    Blank,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingPreferences {
    pub language: LanguagePreference,
    pub theme: ThemePreference,
    pub monitor_id: Option<String>,
    pub background_preset: BackgroundPreset,
    pub starter_layout: StarterLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStatus {
    pub completed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopWidget {
    pub id: i64,
    pub monitor_id: Option<String>,
    pub kind: WidgetKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub locked: bool,
    pub snap_to_grid: bool,
    pub visible: bool,
    pub sort_order: i64,
    #[serde(default)]
    pub clock_settings: Option<ClockWidgetSettings>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroMode {
    Work,
    Break,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroAction {
    Start,
    Pause,
    Reset,
    Skip,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroState {
    pub widget_id: i64,
    pub mode: PomodoroMode,
    pub work_minutes: u16,
    pub break_minutes: u16,
    pub remaining_seconds: i64,
    pub running: bool,
    pub ends_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroPreferences {
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,
}

impl PomodoroPreferences {
    pub fn validate(self) -> Result<Self, String> {
        if self.sound_volume > 100 {
            return Err("Pomodoro ses seviyesi 0 ile 100 arasında olmalıdır.".into());
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroCompletion {
    pub widget_id: i64,
    pub completed_mode: PomodoroMode,
    pub state: PomodoroState,
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

impl WidgetLayout {
    pub fn defaults_for(monitor_id: Option<String>, template: WallpaperTemplate) -> Self {
        let (x, width, height) = match template {
            WallpaperTemplate::Focus => (0.62, 0.34, 0.56),
            WallpaperTemplate::Kanban => (0.52, 0.44, 0.54),
        };
        Self {
            monitor_id,
            template,
            x,
            y: 0.16,
            width,
            height,
            locked: false,
            snap_to_grid: true,
        }
    }

    pub fn validate(self) -> Result<Self, String> {
        if ![self.x, self.y, self.width, self.height]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err("Widget yerleşimi sonlu sayılardan oluşmalıdır.".into());
        }
        if !(0.18..=0.78).contains(&self.width) || !(0.20..=0.78).contains(&self.height) {
            return Err("Widget boyutu izin verilen aralıkta olmalıdır.".into());
        }
        if self.x < 0.0
            || self.y < 0.0
            || self.x + self.width > 1.000_001
            || self.y + self.height > 1.000_001
        {
            return Err("Widget yerleşimi görünür ekran sınırları içinde olmalıdır.".into());
        }
        Ok(self)
    }
}

impl WidgetKind {
    pub fn is_core(self) -> bool {
        matches!(
            self,
            Self::Focus | Self::Kanban | Self::Pomodoro | Self::Clock
        )
    }

    pub fn bundled_packages() -> [Self; 4] {
        [
            Self::Date,
            Self::DailyPoem,
            Self::DailyVerse,
            Self::DailyHadith,
        ]
    }

    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::Kanban => "kanban",
            Self::Pomodoro => "pomodoro",
            Self::Clock => "clock",
            Self::Date => "date",
            Self::DailyPoem => "daily_poem",
            Self::DailyVerse => "daily_verse",
            Self::DailyHadith => "daily_hadith",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "focus" => Ok(Self::Focus),
            "kanban" => Ok(Self::Kanban),
            "pomodoro" => Ok(Self::Pomodoro),
            "clock" => Ok(Self::Clock),
            "date" => Ok(Self::Date),
            "daily_poem" => Ok(Self::DailyPoem),
            "daily_verse" => Ok(Self::DailyVerse),
            "daily_hadith" => Ok(Self::DailyHadith),
            other => Err(format!("Bilinmeyen widget türü: {other}")),
        }
    }

    pub fn default_frame(self) -> (f64, f64, f64, f64) {
        match self {
            Self::Focus => (0.66, 0.16, 0.28, 0.30),
            Self::Kanban => (0.58, 0.16, 0.36, 0.34),
            Self::Pomodoro => (0.05, 0.12, 0.22, 0.26),
            Self::Clock => (0.05, 0.50, 0.17, 0.14),
            Self::Date => (0.28, 0.72, 0.19, 0.13),
            Self::DailyPoem => (0.28, 0.08, 0.25, 0.28),
            Self::DailyVerse => (0.24, 0.48, 0.27, 0.30),
            Self::DailyHadith => (0.03, 0.62, 0.27, 0.28),
        }
    }

    pub fn size_limits(self) -> ((f64, f64), (f64, f64)) {
        match self {
            Self::Focus | Self::Kanban => ((0.10, 0.14), (0.78, 0.78)),
            Self::Pomodoro => ((0.08, 0.12), (0.50, 0.62)),
            Self::Clock => ((0.06, 0.08), (0.46, 0.42)),
            Self::Date => ((0.07, 0.08), (0.52, 0.42)),
            Self::DailyPoem => ((0.10, 0.12), (0.58, 0.66)),
            Self::DailyVerse => ((0.11, 0.13), (0.62, 0.70)),
            Self::DailyHadith => ((0.11, 0.12), (0.60, 0.64)),
        }
    }

    pub fn minimum_pixel_size(self) -> (f64, f64) {
        match self {
            Self::Focus | Self::Kanban => (240.0, 200.0),
            Self::Pomodoro => (190.0, 170.0),
            Self::Clock => (140.0, 95.0),
            Self::Date => (160.0, 95.0),
            Self::DailyPoem => (215.0, 180.0),
            Self::DailyVerse => (230.0, 190.0),
            Self::DailyHadith => (230.0, 180.0),
        }
    }
}

impl WidgetPackage {
    pub fn bundled(kind: WidgetKind, installed: bool) -> Self {
        let ((minimum_width, minimum_height), _) = kind.size_limits();
        Self {
            kind,
            source: if kind.is_core() {
                WidgetPackageSource::Core
            } else {
                WidgetPackageSource::BundledStore
            },
            version: "1.0.0".into(),
            installed: kind.is_core() || installed,
            minimum_width,
            minimum_height,
            permissions: Vec::new(),
        }
    }
}

impl DesktopWidget {
    pub fn defaults_for(monitor_id: Option<String>, kind: WidgetKind, sort_order: i64) -> Self {
        let (mut x, mut y, width, height) = kind.default_frame();
        let offset = (sort_order.rem_euclid(6) as f64) * 0.025;
        x = (x + offset).min(0.985 - width);
        y = (y + offset).min(0.985 - height);
        Self {
            id: 0,
            monitor_id,
            kind,
            x,
            y,
            width,
            height,
            locked: false,
            snap_to_grid: true,
            visible: true,
            sort_order,
            clock_settings: (kind == WidgetKind::Clock).then(ClockWidgetSettings::default),
        }
    }

    pub fn validate(mut self) -> Result<Self, String> {
        if ![self.x, self.y, self.width, self.height]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err("Widget yerleşimi sonlu sayılardan oluşmalıdır.".into());
        }
        let ((min_width, min_height), (max_width, max_height)) = self.kind.size_limits();
        if !(min_width..=max_width).contains(&self.width)
            || !(min_height..=max_height).contains(&self.height)
        {
            return Err("Widget boyutu izin verilen aralıkta olmalıdır.".into());
        }
        if self.x < 0.0
            || self.y < 0.0
            || self.x + self.width > 1.000_001
            || self.y + self.height > 1.000_001
        {
            return Err("Widget yerleşimi görünür ekran sınırları içinde olmalıdır.".into());
        }
        self.clock_settings = if self.kind == WidgetKind::Clock {
            Some(self.clock_settings.unwrap_or_default().validate()?)
        } else {
            None
        };
        Ok(self)
    }

    pub fn validate_for_viewport(
        &self,
        viewport_width: f64,
        viewport_height: f64,
    ) -> Result<(), String> {
        if viewport_width <= 0.0 || viewport_height <= 0.0 {
            return Err("Monitör ölçüsü geçersiz.".into());
        }
        let ((base_width, base_height), _) = self.kind.size_limits();
        let (pixel_width, pixel_height) = self.kind.minimum_pixel_size();
        let min_width = base_width.max(pixel_width / viewport_width);
        let min_height = base_height.max(pixel_height / viewport_height);
        if self.width + f64::EPSILON < min_width || self.height + f64::EPSILON < min_height {
            return Err("Widget boyutu hedef monitör için çok küçük.".into());
        }
        Ok(())
    }
}

impl PomodoroMode {
    pub fn as_database_value(self) -> &'static str {
        match self {
            Self::Work => "work",
            Self::Break => "break",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "work" => Ok(Self::Work),
            "break" => Ok(Self::Break),
            other => Err(format!("Bilinmeyen Pomodoro modu: {other}")),
        }
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
            Self::Porcelain => "porcelain",
            Self::Arctic => "arctic",
            Self::Linen => "linen",
            Self::MorningMist => "morning_mist",
        }
    }

    pub fn from_database_value(value: &str) -> Result<Self, String> {
        match value {
            "folded_horizon" => Ok(Self::FoldedHorizon),
            "midnight" => Ok(Self::Midnight),
            "graphite" => Ok(Self::Graphite),
            "ember" => Ok(Self::Ember),
            "porcelain" => Ok(Self::Porcelain),
            "arctic" => Ok(Self::Arctic),
            "linen" => Ok(Self::Linen),
            "morning_mist" => Ok(Self::MorningMist),
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

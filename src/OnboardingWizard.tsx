import { useEffect, useState } from "react";
import appIcon from "./assets/interactivebackground-icon.png";
import { useI18n } from "./i18n";
import type { AppSettings, BackgroundPreset, LanguagePreference, MonitorInfo, OnboardingPreferences, StarterLayout, ThemePreference } from "./types";
import type { TranslationKey } from "./i18n/locales/en";
import { BACKGROUND_PRESETS } from "./backgroundPresets";
import { BackgroundArtwork } from "./BackgroundArtwork";

type Props = {
  settings: AppSettings;
  monitors: MonitorInfo[];
  autoStartEnabled: boolean;
  initialBackgroundPreset: BackgroundPreset;
  canDismiss: boolean;
  onDismiss: () => void;
  onComplete: (preferences: OnboardingPreferences, autoStart: boolean) => Promise<void>;
};

export function OnboardingWizard({ settings, monitors, autoStartEnabled, initialBackgroundPreset, canDismiss, onDismiss, onComplete }: Props) {
  const [step, setStep] = useState(0);
  const [language, setLanguage] = useState<LanguagePreference>(settings.language);
  const [theme, setTheme] = useState<ThemePreference>(settings.theme);
  const [monitorId, setMonitorId] = useState<string | null>(settings.monitorId);
  const [backgroundPreset, setBackgroundPreset] = useState<BackgroundPreset>(initialBackgroundPreset);
  const [starterLayout, setStarterLayout] = useState<StarterLayout>("focus");
  const [autoStart, setAutoStart] = useState(autoStartEnabled);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const { t } = useI18n(language);

  useEffect(() => {
    if (monitorId || monitors.length === 0) return;
    setMonitorId(monitors.find((monitor) => monitor.isPrimary)?.id ?? monitors[0].id);
  }, [monitorId, monitors]);
  useEffect(() => setAutoStart(autoStartEnabled), [autoStartEnabled]);

  async function finish() {
    setSaving(true);
    setError("");
    try {
      await onComplete({ language, theme, monitorId, backgroundPreset, starterLayout }, autoStart);
    } catch (reason) {
      setError(String(reason));
      setSaving(false);
    }
  }

  const steps = [t("onboarding.stepWelcome"), t("onboarding.stepLook"), t("onboarding.stepWorkspace"), t("onboarding.stepReady")];

  return <div className="onboarding-backdrop" role="presentation">
    <section className="onboarding-dialog" role="dialog" aria-modal="true" aria-labelledby="onboarding-title">
      <aside className="onboarding-rail">
        <div className="onboarding-brand"><img src={appIcon} alt="" aria-hidden="true" /><strong>interactivebackground</strong></div>
        <ol>{steps.map((label, index) => <li className={index === step ? "active" : index < step ? "done" : ""} key={label}><span>{index < step ? "✓" : index + 1}</span>{label}</li>)}</ol>
        <p>{t("onboarding.localNote")}</p>
      </aside>

      <div className="onboarding-main">
        {canDismiss && <button className="onboarding-close" type="button" onClick={onDismiss} aria-label={t("onboarding.close")}>×</button>}

        {step === 0 && <div className="onboarding-step">
          <p className="eyebrow">{t("onboarding.eyebrow")}</p>
          <h1 id="onboarding-title">{t("onboarding.welcomeTitle")}</h1>
          <p className="onboarding-lead">{t("onboarding.welcomeBody")}</p>
          <div className="onboarding-feature-grid">
            <article><span>▦</span><b>{t("onboarding.featureWidgets")}</b><small>{t("onboarding.featureWidgetsBody")}</small></article>
            <article><span>◫</span><b>{t("onboarding.featureDesktop")}</b><small>{t("onboarding.featureDesktopBody")}</small></article>
            <article><span>⌁</span><b>{t("onboarding.featurePrivate")}</b><small>{t("onboarding.featurePrivateBody")}</small></article>
          </div>
          <label className="onboarding-field"><span>{t("language.label")}</span><select value={language} onChange={(event) => setLanguage(event.target.value as LanguagePreference)}><option value="system">{t("language.system")}</option><option value="tr">{t("language.tr")}</option><option value="en">{t("language.en")}</option></select></label>
        </div>}

        {step === 1 && <div className="onboarding-step">
          <p className="eyebrow">{t("onboarding.stepLook")}</p>
          <h1 id="onboarding-title">{t("onboarding.lookTitle")}</h1>
          <p className="onboarding-lead">{t("onboarding.lookBody")}</p>
          <h3>{t("theme.label")}</h3>
          <div className="onboarding-choice-grid three">
            {(["system", "light", "dark"] as ThemePreference[]).map((value) => <button className={theme === value ? "selected" : ""} onClick={() => setTheme(value)} key={value}><span className={`theme-chip theme-${value}`} /><b>{t(`theme.${value}` as "theme.system" | "theme.light" | "theme.dark")}</b></button>)}
          </div>
          <h3>{t("background.title")}</h3>
          <div className="onboarding-choice-grid four">
            {BACKGROUND_PRESETS.map((preset) => <button className={backgroundPreset === preset ? "selected" : ""} onClick={() => setBackgroundPreset(preset)} key={preset}><BackgroundArtwork compact preset={preset} /><b>{t(`background.${preset}` as TranslationKey)}</b></button>)}
          </div>
        </div>}

        {step === 2 && <div className="onboarding-step">
          <p className="eyebrow">{t("onboarding.stepWorkspace")}</p>
          <h1 id="onboarding-title">{t("onboarding.workspaceTitle")}</h1>
          <p className="onboarding-lead">{t("onboarding.workspaceBody")}</p>
          <label className="onboarding-field"><span>{t("monitor.label")}</span><select value={monitorId ?? ""} onChange={(event) => setMonitorId(event.target.value || null)}>{monitors.map((monitor) => <option value={monitor.id} key={monitor.id}>{monitor.id.startsWith("browser:") ? t("monitor.browserDisplay") : monitor.name}{monitor.isPrimary ? ` · ${t("monitor.primary")}` : ""} — {monitor.width}×{monitor.height}</option>)}</select></label>
          <div className="onboarding-layout-grid">
            {(["focus", "planning", "blank"] as StarterLayout[]).map((layout) => <button className={starterLayout === layout ? "selected" : ""} onClick={() => setStarterLayout(layout)} key={layout}><span className={`layout-illustration layout-${layout}`}><i /><i /><i /></span><b>{t(`onboarding.layout.${layout}` as "onboarding.layout.focus" | "onboarding.layout.planning" | "onboarding.layout.blank")}</b><small>{t(`onboarding.layout.${layout}Body` as "onboarding.layout.focusBody" | "onboarding.layout.planningBody" | "onboarding.layout.blankBody")}</small></button>)}
          </div>
          <label className="switch-row onboarding-autostart"><input type="checkbox" checked={autoStart} onChange={(event) => setAutoStart(event.target.checked)} /><span><b>{t("autostart.label")}</b><small>{t("onboarding.autostartConsent")}</small></span></label>
        </div>}

        {step === 3 && <div className="onboarding-step onboarding-ready">
          <div className="ready-mark">✓</div>
          <p className="eyebrow">{t("onboarding.stepReady")}</p>
          <h1 id="onboarding-title">{t("onboarding.readyTitle")}</h1>
          <p className="onboarding-lead">{t("onboarding.readyBody")}</p>
          <div className="onboarding-summary">
            <span><b>{t("language.label")}</b>{t(`language.${language}` as "language.system" | "language.tr" | "language.en")}</span>
            <span><b>{t("theme.label")}</b>{t(`theme.${theme}` as "theme.system" | "theme.light" | "theme.dark")}</span>
            <span><b>{t("background.title")}</b>{t(`background.${backgroundPreset}` as TranslationKey)}</span>
            <span><b>{t("onboarding.layoutLabel")}</b>{t(`onboarding.layout.${starterLayout}` as "onboarding.layout.focus" | "onboarding.layout.planning" | "onboarding.layout.blank")}</span>
          </div>
          <div className="shortcut-card"><kbd>Ctrl</kbd><b>+</b><kbd>Alt</kbd><b>+</b><kbd>Space</kbd><span>{t("onboarding.shortcutHelp")}</span></div>
          {error && <p className="error-message" role="alert">{error}</p>}
        </div>}

        <footer className="onboarding-footer">
          <span>{t("onboarding.progress", { current: step + 1, total: steps.length })}</span>
          <div>{step > 0 && <button className="onboarding-secondary" type="button" disabled={saving} onClick={() => setStep((value) => value - 1)}>{t("onboarding.back")}</button>}{step < steps.length - 1 ? <button className="onboarding-primary" type="button" onClick={() => setStep((value) => value + 1)}>{t("onboarding.continue")}</button> : <button className="onboarding-primary" type="button" disabled={saving || !monitorId} onClick={() => void finish()}>{saving ? t("onboarding.saving") : t("onboarding.finish")}</button>}</div>
        </footer>
      </div>
    </section>
  </div>;
}

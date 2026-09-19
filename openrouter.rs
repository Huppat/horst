/// OpenRouter-Integration für Horst.
///
/// Lädt verfügbare Modelle, prüft deren Status und verwaltet den API-Key.
/// Config wird in ~/.config/horst/config.json gespeichert.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

// ── Persistente Konfiguration ────────────────────────────────────────────────

/// Lokal- und OpenRouter-Einstellungen sind getrennte Felder, damit der Wechsel
/// zwischen beiden Modi den jeweils anderen nicht überschreibt oder kaputt macht.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorstConfig {
    /// "local" oder "openrouter" — der zuletzt aktive Modus.
    #[serde(default = "default_mode")]
    pub mode: String,

    /// API-Adresse des lokalen llama.cpp-Servers.
    #[serde(default = "default_api_base")]
    pub local_api_base: String,
    /// Zuletzt gewähltes lokales Modell (meist leer — der Server hat nur eins geladen).
    #[serde(default)]
    pub local_model: String,

    /// Adresse eines dedizierten Embedding-Servers (leer = Chat-Server mitverwenden,
    /// siehe `AppConfig::embedding_url`). Unabhängig vom Modus (local/openrouter) —
    /// eine feste Infrastruktur-Adresse, kein Teil des Chat-Umschaltens.
    #[serde(default)]
    pub embedding_api_base: String,

    /// OpenRouter API-Key.
    #[serde(default)]
    pub openrouter_api_key: String,
    /// Zuletzt gewähltes OpenRouter-Modell.
    #[serde(default)]
    pub openrouter_model: String,

    /// Override-Temperatur, für beide Modi gemeinsam. None = der Server entscheidet.
    #[serde(default)]
    pub temperature: Option<f32>,

    /// Sampling-Parameter top_p, für beide Modi gemeinsam.
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    /// Sampling-Parameter top_k, für beide Modi gemeinsam.
    #[serde(default = "default_top_k")]
    pub top_k: u32,

    /// Ab wie viel Prozent des Server-Kontextfensters der Verlauf verdichtet wird.
    #[serde(default = "default_compact_percent")]
    pub compact_percent: u8,

    // ── Alte Feldnamen (vor der Trennung von lokal/OpenRouter) — nur zum Einlesen
    // bestehender config.json-Dateien, werden beim Speichern nicht mehr geschrieben. ──
    #[serde(rename = "api_key", default, skip_serializing)]
    legacy_api_key: String,
    #[serde(rename = "model", default, skip_serializing)]
    legacy_model: String,
    #[serde(rename = "api_base", default, skip_serializing)]
    legacy_api_base: String,
}

fn default_mode() -> String { "local".to_string() }
fn default_api_base() -> String { "http://127.0.0.1:8080".to_string() }
fn default_compact_percent() -> u8 { 75 }
fn default_top_p() -> f32 { 0.95 }
fn default_top_k() -> u32 { 40 }

impl Default for HorstConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            local_api_base: default_api_base(),
            local_model: String::new(),
            embedding_api_base: String::new(),
            openrouter_api_key: String::new(),
            openrouter_model: String::new(),
            temperature: None,
            top_p: default_top_p(),
            top_k: default_top_k(),
            compact_percent: default_compact_percent(),
            legacy_api_key: String::new(),
            legacy_model: String::new(),
            legacy_api_base: String::new(),
        }
    }
}

/// Fragt die Standard-Temperatur des llama.cpp-Servers über /props ab.
/// Gibt None zurück, wenn der Server das nicht liefert (z.B. OpenRouter).
pub async fn fetch_server_temperature(api_base: &str) -> Option<f32> {
    let url = format!("{}/props", api_base.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    json.get("default_generation_settings")?
        .get("params")?
        .get("temperature")?
        .as_f64()
        .map(|t| t as f32)
}

/// Fragt die Kontextfenster-Größe (n_ctx, in Tokens) des llama.cpp-Servers über /props ab.
/// Gibt None zurück, wenn der Server das nicht liefert (z.B. OpenRouter, ältere Version).
pub async fn fetch_server_context_size(api_base: &str) -> Option<usize> {
    let url = format!("{}/props", api_base.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    json.get("default_generation_settings")?
        .get("n_ctx")?
        .as_u64()
        .filter(|n| *n > 0)
        .map(|n| n as usize)
}

impl HorstConfig {
    pub fn load() -> Self {
        let mut cfg: HorstConfig = match std::fs::read_to_string(config_path()) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => HorstConfig::default(),
        };
        cfg.migrate_legacy_fields();
        // Bei jedem Neustart mit dem lokalen Modell beginnen — OpenRouter ist nur ein
        // Einstieg für Nutzer ohne eigene lokale KI, kein dauerhafter Standard. Der
        // Nutzer kann pro laufender Sitzung im Einstellungs-Panel manuell auf
        // OpenRouter wechseln; das gilt dann nur bis zum nächsten Neustart.
        cfg.mode = "local".to_string();
        cfg
    }

    /// Alte config.json-Dateien hatten ein gemeinsames model/api_key/api_base-Feld
    /// für beide Modi. Übernimmt sie beim ersten Laden in die getrennten neuen Felder.
    fn migrate_legacy_fields(&mut self) {
        if !self.legacy_api_base.is_empty() && self.local_api_base == default_api_base() {
            self.local_api_base = std::mem::take(&mut self.legacy_api_base);
        }
        if !self.legacy_model.is_empty() {
            if self.mode == "openrouter" && self.openrouter_model.is_empty() {
                self.openrouter_model = std::mem::take(&mut self.legacy_model);
            } else if self.mode != "openrouter" && self.local_model.is_empty() {
                self.local_model = std::mem::take(&mut self.legacy_model);
            }
        }
        if !self.legacy_api_key.is_empty() && self.openrouter_api_key.is_empty() {
            self.openrouter_api_key = std::mem::take(&mut self.legacy_api_key);
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        info!("Konfiguration gespeichert: {}", path.display());
        Ok(())
    }
}

fn config_path() -> PathBuf {
    crate::config::data_dir().join("config.json")
}

// ── OpenRouter API-Typen ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
struct OrModelsResponse {
    data: Vec<OrModel>,
}

#[derive(Debug, Deserialize, Clone)]
struct OrModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    context_length: Option<u64>,
    /// Von OpenRouter deklarierte Parameter-Unterstützung (z.B. "tools").
    #[serde(default)]
    supported_parameters: Vec<String>,
    #[serde(default)]
    pricing: Option<OrPricing>,
    #[serde(default)]
    top_provider: Option<OrTopProvider>,
}

/// Preise als Strings pro Token (z.B. "0.00000003"). "0" bzw. fehlend = kostenlos.
#[derive(Debug, Deserialize, Clone, Default)]
struct OrPricing {
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    completion: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct OrTopProvider {
    /// Maximale Ausgabelänge pro Anfrage. Bester verfügbarer Näherungswert dafür,
    /// wie weit ein Gratis-Modell in einem Rutsch kommt (OpenRouter legt kein echtes
    /// Token-Tageskontingent pro Modell offen).
    #[serde(default)]
    max_completion_tokens: Option<u64>,
}

impl OrModel {
    fn supports_tools(&self) -> bool {
        self.supported_parameters.iter().any(|p| p == "tools")
    }

    /// Preis pro 1M Ausgabe-Tokens in US-Dollar. `None`, wenn kein Preis bekannt ist
    /// (z.B. der variabel bepreiste Auto-Router `openrouter/auto`).
    fn price_per_million(&self) -> Option<f64> {
        let p = self.pricing.as_ref()?;
        let prompt = p.prompt.as_deref().and_then(|s| s.parse::<f64>().ok());
        let completion = p.completion.as_deref().and_then(|s| s.parse::<f64>().ok());
        match (prompt, completion) {
            (Some(pr), Some(co)) if pr >= 0.0 && co >= 0.0 => Some(co * 1_000_000.0),
            _ => None, // negativer/fehlender Preis = variabel (Auto-Router)
        }
    }

    fn is_free(&self) -> bool {
        matches!(self.price_per_million(), Some(v) if v == 0.0)
            && self
                .pricing
                .as_ref()
                .and_then(|p| p.prompt.as_deref())
                .and_then(|s| s.parse::<f64>().ok())
                == Some(0.0)
    }

    fn max_out(&self) -> u64 {
        self.top_provider
            .as_ref()
            .and_then(|t| t.max_completion_tokens)
            .unwrap_or(0)
    }
}

// ── Öffentliche Modell-Info (ans Electron geschickt) ─────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context: String,
    /// Gruppe fürs Dropdown: 0 = gratis + tool-fähig, 1 = günstig bezahlt + tool-fähig,
    /// 2 = übrige (teurer oder ohne Tool-Unterstützung).
    pub group: u8,
    /// Kostenlos nutzbar.
    pub free: bool,
    /// Unterstützt Tool-Aufrufe (Horst braucht das, um wirklich zu handeln statt die
    /// Ausführung nur im Text nachzuspielen).
    pub tools: bool,
    /// Kurzes Preisschild fürs Badge: "GRATIS", "$0,03/1M" oder "variabel".
    pub price_label: String,
}

/// Bekannte Free-Modelle, die trotz Tool-Support unzuverlässig sind (häufige 429,
/// leere Antworten, Disconnects). Sie werden aus der Liste gefiltert.
const BLACKLISTED_FREE_MODELS: &[&str] = &[
    "google/gemma-4-31b-it:free",
    "google/gemini-4-26b-a4b-it:free",
    "qwen/qwen3-next-80b-a3b-instruct:free",
    "meta-llama/llama-3.3-70b-instruct:free",
    "qwen/qwen3-coder:free",
];

/// Lädt die Liste der verfügbaren Modelle von OpenRouter.
pub async fn fetch_models(api_key: &str) -> Result<Vec<ModelInfo>> {
    if api_key.trim().is_empty() {
        return Err(anyhow!("Kein API-Key angegeben"));
    }

    let client = reqwest::Client::builder()
        .user_agent("Horst/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client
        .get("https://openrouter.ai/api/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await?
        .json::<OrModelsResponse>()
        .await?;

    let mut raw = resp.data;

    // Bekannte unzuverlässige Free-Modelle herausfiltern.
    raw.retain(|m| !BLACKLISTED_FREE_MODELS.contains(&m.id.as_str()));

    // Sortierung: erst nach Gruppe (gratis+Tools → günstig+Tools → Rest), dann innerhalb.
    raw.sort_by(|a, b| group_of(a).cmp(&group_of(b)).then_with(|| within_group_cmp(a, b)));

    let models = raw.into_iter().map(build_model_info).collect();
    Ok(models)
}

/// Gruppe eines Modells fürs Dropdown (siehe ModelInfo::group).
fn group_of(m: &OrModel) -> u8 {
    match (m.supports_tools(), m.is_free()) {
        (true, true) => 0,
        (true, false) => 1,
        (false, _) => 2,
    }
}

/// Sortierung innerhalb einer Gruppe. Setzt voraus, dass a und b in derselben Gruppe liegen.
fn within_group_cmp(a: &OrModel, b: &OrModel) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match group_of(a) {
        // Gratis + Tools: nach "Durchhaltevermögen" (max Ausgabe) absteigend. Der
        // Auto-Router openrouter/free wandert ans Ende der Gruppe (wählt mal Nicht-Tool-
        // Modelle), bleibt aber drin.
        0 => {
            let a_last = a.id == "openrouter/free";
            let b_last = b.id == "openrouter/free";
            a_last
                .cmp(&b_last)
                .then_with(|| b.max_out().cmp(&a.max_out()))
                .then_with(|| a.id.cmp(&b.id))
        }
        // Günstig bezahlt + Tools: nach Preis aufsteigend; variabel bepreiste (Auto-Router)
        // ans Ende.
        1 => {
            let pa = a.price_per_million().unwrap_or(f64::MAX);
            let pb = b.price_per_million().unwrap_or(f64::MAX);
            pa.partial_cmp(&pb)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        }
        // Rest: alphabetisch.
        _ => a.id.to_lowercase().cmp(&b.id.to_lowercase()),
    }
}

/// Baut die ans Frontend geschickte Modell-Info aus einem rohen OpenRouter-Modell.
fn build_model_info(m: OrModel) -> ModelInfo {
    let free = m.is_free();
    let tools = m.supports_tools();
    let group = group_of(&m);
    let price_label = if free {
        "GRATIS".to_string()
    } else {
        match m.price_per_million() {
            Some(v) if v < 1.0 => format!("${:.3}/1M", v).replace('.', ","),
            Some(v) => format!("${:.2}/1M", v).replace('.', ","),
            None => "variabel".to_string(),
        }
    };
    let context = fmt_context(m.context_length);
    ModelInfo {
        id: m.id,
        name: m.name.unwrap_or_default(),
        description: m.description.unwrap_or_default().chars().take(100).collect(),
        context,
        group,
        free,
        tools,
        price_label,
    }
}

/// Kontextlänge menschenlesbar: 1048576 → "1M", 131072 → "131K".
fn fmt_context(len: Option<u64>) -> String {
    match len {
        Some(c) if c >= 1_000_000 => format!("{}M", c / 1_000_000),
        Some(c) if c >= 1_000 => format!("{}K", c / 1_000),
        Some(c) => c.to_string(),
        None => "-".to_string(),
    }
}

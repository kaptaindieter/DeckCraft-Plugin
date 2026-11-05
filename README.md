# DeckCraft-Plugin — Craft Suite Plugin for DeckCraft

Eigenstaendiges Craft-Suite-Plugin fuer Vanilla DeckCraft: Integrierter BYOK AI-Agent mit Praesentations- & Folien-Commands, dynamischer Cloud- & Auto-Save-Sync (frei waehlbarer Ordner) und Aurora/Dark/Light Farbschema-System.

> **Wichtiger Hinweis zur Architektur:**
> Dieses Repository enthaelt **ausschliesslich das eigenstaendige Plugin** (die neuen Rust-Module, AI-Icons, den Hook-Patch und den Installer) und **nicht** den gesamten Vanilla-Quellcode von DeckCraft. Jeder Nutzer laedt sich einfach die originale Vanilla-Version von [https://github.com/storytold/deckcraft](https://github.com/storytold/deckcraft) herunter und installiert anschliessend dieses Plugin darueber.

---

## Features des Plugins

### 1. BYOK AI-Agent & Command-Bridge (`artcraft_agent.rs`)
- **Bring Your Own Key (BYOK)**: Jeder Nutzer kann in der AI-Sidebar ueber **`+ API`** bzw. **`Manage APIs`** seine eigenen API-Keys und Endpunkte hinterlegen (OpenAI / GPT, Google Gemini, DeepSeek, Mistral, Qwen, lokales Ollama oder jeder OpenAI-kompatible Server).
- **Keine hartcodierten API-Keys**: Alle Zugangsdaten werden ausschliesslich lokal im jeweiligen Benutzerprofil (`~/.craft-suite/shared-ai-config.json` bzw. `%APPDATA%\CraftSuite\shared-ai-config.json`) gespeichert.
- **Direkte App-Steuerung ueber Commands**:
  - Unterstuetzte Commands: `deck.createPresentation`, `deck.addRichSlide`, `deck.addTableSlide`, `slide.insert`, `shape.insertTextBox`, `get_slides`, `inspect_presentation`, `save_presentation`

### 2. Dynamisches Cloud- & Auto-Save-Sync (`artcraft_drive.rs`)
- **Frei waehlbarer Sync-Ordner fuer jeden Endnutzer**: Ueber einen Klick auf das Cloud-Status-Badge in der oberen rechten Aktionsleiste oeffnet sich das Konfigurationsfenster **`☁️ Cloud & Auto-Save Sync`**.
- **Anbieter & Ordner selbst waehlen**: Unterstuetzt Google Drive, OneDrive, Nextcloud, Dropbox oder jeden beliebigen lokalen Ordner ueber den nativen Ordner-Auswahldialog (**`📂 Choose Folder`**).
- **Lokale Persistenz**: Die gewaehlte Pfad-Konfiguration wird lokal unter `~/.craft-suite/cloud-config.json` gespeichert (keine fremden Nutzerpfade im Code).

### 3. Farbschema-System (`Aurora`, `Dark`, `Light`) & UI-Design (`artcraft_theme.rs`)
- **Neues Farbschema-System**: Erweitert DeckCraft um den kosmischen **Aurora**-Modus (inkl. Regenbogen-Akzentlinie) zusaetzlich zu **Dark** und **Light**.
- **Einheitliche Aktionsleiste & Theme-Switcher**: Standardisierte 28px Top-Right Action Bar (`✓ Auto-Save`, `● Cloud-Status`, `✨ AI Agent`) sowie Theme-Umschalter in der Fusszeile.

---

## Kompatibilitaet mit der Vanilla-Version

- **Originales Upstream-Repository**: [https://github.com/storytold/deckcraft](https://github.com/storytold/deckcraft)
- **Getesteter Vanilla-Basis-Commit**: `19dceffd0bf3cbc79f9aaaf23fe3e05932fb434d` (*"pptx: index media, layouts, sections and table styles on export (#1761)"*)

---

## Installation (Schritt fuer Schritt)

### Voraussetzungen
- **Git** und **Python 3.8+** (oder PowerShell unter Windows)
- **Rust Toolchain** (`cargo`, Rust 1.80+)

### 1. Originale Vanilla-Version herunterladen
```bash
git clone https://github.com/storytold/deckcraft.git
cd deckcraft
# Optional: Exakten getesteten Vanilla-Stand auschecken
git checkout 19dceffd0bf3cbc79f9aaaf23fe3e05932fb434d
cd ..
```

### 2. Dieses Plugin herunterladen & installieren
```bash
git clone https://github.com/kaptaindieter/DeckCraft-Plugin.git
cd DeckCraft-Plugin

# Plugin auf die Vanilla-Version anwenden (und optional direkt kompilieren):
python install_plugin.py ../deckcraft --build
```

Oder unter Windows per PowerShell:
```powershell
.\install_plugin.ps1 -TargetDir ..\deckcraft -Build
```

Die fertige ausfuehrbare Datei liegt anschliessend unter:
`../deckcraft/target/release/deckcraft.exe` (bzw. `target/release/deckcraft` unter Linux/macOS).

---

## Deinstallation (Vanilla-Zustand wiederherstellen)

Falls du das Vanilla-Repository wieder in den 100% originalen Zustand zurueckversetzen moechtest:

```bash
python uninstall_plugin.py ../deckcraft
```

Oder per PowerShell:
```powershell
.\uninstall_plugin.ps1 -TargetDir ..\deckcraft
```

---

## Paketstruktur des Plugins

- `plugin/src/artcraft_agent.rs`
- `plugin/src/artcraft_mcp.rs`
- `plugin/src/artcraft_drive.rs`
- `plugin/src/artcraft_theme.rs`
- `plugin/assets/ai_icons/` — Icons fuer die KI-Modell-Anbieter (OpenAI, Gemini, DeepSeek, Llama, Mistral, Qwen, AI)
- `patches/vanilla-hooks.patch` — Minimaler Integrations-Patch fuer die Vanilla-Dateien:
- `apps/deckcraft-web/index.html`
- `crates/ui-egui/Cargo.toml`
- `crates/ui-egui/src/app.rs`
- `crates/ui-egui/src/app/frame.rs`
- `crates/ui-egui/src/backstage/files.rs`
- `crates/ui-egui/src/dialogs/about.rs`
- `crates/ui-egui/src/dialogs/body.rs`
- `crates/ui-egui/src/dialogs/preferences.rs`
- `crates/ui-egui/src/lib.rs`
- `crates/ui-egui/src/ribbon/title_bar.rs`
- `crates/ui-egui/src/status.rs`
- `crates/ui-egui/src/theme.rs`
- `config/cloud-config.example.json` — Beispiel-Vorlage fuer die Endnutzer-Cloud-Konfiguration
- `config/shared-ai-config.example.json` — Beispiel-Vorlage fuer die eigene BYOK-API-Konfiguration
- `config/UPSTREAM_VERSION.json` — Versions- & Commit-Metadaten der Vanilla-Basis
- `install_plugin.py` / `install_plugin.ps1` — Automatischer Plugin-Installer
- `uninstall_plugin.py` / `uninstall_plugin.ps1` — Automatischer Plugin-Uninstaller

---

## Lizenz

MIT License — siehe [LICENSE](LICENSE).

# Horst - KI-Assistent für Einsteiger

Horst ist dein persönlicher Linux-Assistent — kostenlos, einfach, auf Deutsch. Er hilft dir bei allen Linux-Themen: Software installieren, Dateien verwalten, System-Einstellungen, Programmieren und mehr.

## Was Horst kann

- **Kaputte Updates retten** — Abgebrochene oder fehlerhafte System-Updates reparieren, Abhängigkeiten reparieren
- **Fenster bedienen** — Programme mit Knöpfen, Formularen und Dialogen steuern
- **D-Bus steuern** — Lautstärke, Medienplayer, Desktop-Dienste, Systemdienste
- **Internet recherchieren** — Suchen, Webseiten lesen, Treffer aufrufen, Bilder herunterladen
- **Code schreiben** — Programme programmieren, testen, Fehler beheben
- **System und Dienste** — Services starten/stoppen, Konfigurationen ändern, Benutzer verwalten
- **Hardware erkennen** — Festplatten, Grafikkarten, Drucker, Netzwerkgeräte
- **Sicherheit** — Firewall, SSH, Berechtigungen, Updates sichern
- **Dateien durchsuchen** — Nach Namen und Inhalt suchen, große Projekte durchforsten
- **Pläne erstellen** — Komplexe Aufgaben in Schritte zerlegen und abarbeiten

## Installation

**[📥 Horst-Installer 0.1.238 herunterladen (ZIP)](https://github.com/Huppat/horst/releases/download/Agent/Horst-Installer-0.1.238.zip)**

Entpacke den Installer und führe ihn aus:

```bash
# Installer ausführbar machen
chmod +x Horst-Installer-0.1.238.zip

# Horst installieren
./Horst-Installer-0.1.238.zip
```

## Startoptionen

- `horst` — Normaler Start mit Electron-Oberfläche
- `horst --help` — Alle Optionen anzeigen
- `horst --version` — Versionsnummer anzeigen
- `horst --debug` — Debug-Modus für Fehleranalyse

## Nutzung

Horst startet mit einer Electron-Oberfläche. Du stellst Fragen oder gibst Aufgaben — Horst führt sie aus.

**Beispiele:**
- "Mein Update ist abgebrochen, reparier das"
- "Welche Festplatten sind angeschlossen?"
- "Stelle die Lautstärke auf 75%"
- "Suche nach einer Anleitung für Git"
- "Erstelle eine Webseite für mein Projekt"
- "Installiere Firefox und konfiguriere die Firewall"

## Spenden

Horst ist kostenlos und Open Source. Wenn du Horst unterstützen möchtest, kannst du gerne spenden:

[**Spenden via PayPal**](https://paypal.me/huppat)

## Lizenz

Horst ist freie Software.

## Beta-Test

Du möchtest Horst als Beta-Tester testen? Lade dir den Installer herunter und schreib uns eine Nachricht auf GitHub!

## Bekannte Fehler

- **Linux Mint 22:** Ein Bug kann dazu führen, dass Horst das Betriebssystem herunterfährt oder neu startet.
- **Minimieren-Button:** Der Minimieren-Button in der Electron-Oberfläche funktioniert nicht.

## Einschränkungen

- **Externe Modelle:** Zurzeit sind externe Sprachmodelle nur über OpenRouter verfügbar.

## Tipps

- **Schließen-Button:** Der Schließen-Button schließt Horst nicht vollständig, sondern minimiert ihn in den Task-Manager.
- **Horst vollständig beenden:** Klicke mit der rechten Maustaste auf das Horst-Icon in der Taskleiste und wähle „Beenden".
- **Arch Linux:** Horst läuft unter Arch Linux (bzw. CachyOS) am besten.

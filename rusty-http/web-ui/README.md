# Rusty Arena

Statische Oberfläche ohne Build-Schritt oder externe Abhängigkeiten.

```sh
python3 web-ui/server.py
```

Öffnen: http://127.0.0.1:8080. Rust-API und Roboterspiel müssen laufen.
Anderes API-Ziel: `python3 web-ui/server.py --api http://127.0.0.1:3000 --port 8080`.

Der Python-Server liefert die Dateien aus und reicht die API-Routen (z. B. `/create`) an die API weiter.
`POST /robot_state` wird unverändert weitergeleitet. SSE wird direkt durchgereicht.
Alternativ können die drei Frontend-Dateien mit einem eigenen gleichwertigen
Reverse-Proxy ausgeliefert werden.

Die UI unterstützt optional `GET /robot_list` als JSON-Array von Namen und lädt
deren Zustände automatisch. Zusätzlich werden Roboter über SSE bekannt.
Vorhandene Roboter stehen im Dropdown der Steuerzentrale. Bekannte Namen werden
lokal gespeichert, ihre Zustände beim nächsten Laden neu abgefragt. Änderungen
kommen über die SSE-Ereignisse `Created`, `Changed`, `Removed` und `Rename`.
Die Karte lädt `/board_dimensions` (`{x, y}`) und `/chargepads` (`[{x, y}, …]`).
Die Spielfeldgrenze folgt dem um den Ursprung zentrierten Koordinatensystem des Spiels.
Ladefelder sind mit einem goldenen Blitz markiert. „Gesamtansicht“ passt das ganze
Spielfeld in die Ansicht ein. Einzelne Wände innerhalb des Spielfelds liefert die API nicht.
SSE bietet keinen Ereignis-Replay; Änderungen während einer Unterbrechung können fehlen.

Steps werden für jede Richtung an die API gesendet. Im aktuellen Spielcode
wertet nur `up` die Schrittzahl aus. Teamzuordnung und Reichweite bestimmt das Spiel.
Ein HTTP-Erfolg bestätigt den Befehlsversand; tatsächliche Änderungen bestätigt SSE.

Die Oberfläche verwendet direkte Routen (`/create`, `/event`, …) und kann auch
von Rust ausgeliefert werden. Zustände werden per `POST /robot_state` mit JSON abgefragt.

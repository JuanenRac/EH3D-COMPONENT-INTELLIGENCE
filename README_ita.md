<p align="center">
  <img src="images/EH3D_COMPONENT_INTELLIGENCE_BANNER.svg" alt="EH3D-COMPONENT-INTELLIGENCE banner" width="100%">
</p>

# <img src="images/brand/png/icon-64.png" alt="" width="40" height="40" align="absmiddle"> EH3D-COMPONENT-INTELLIGENCE

<p align="center"><a href="README.md">🇺🇸 English</a> | <a href="README_spa.md">🇪🇸 Español</a> | <a href="README_fra.md">🇫🇷 Français</a> | 🇮🇹 <b>Italiano</b> | <a href="README_deu.md">🇩🇪 Deutsch</a> | <a href="README_zho.md">🇨🇳 中文</a> | <a href="README_jpn.md">🇯🇵 日本語</a> | <a href="README_por.md">🇵🇹 Português</a></p>

### EH3D-COMPONENT-INTELLIGENCE: una base di dati tecnica di componenti elettronici in cui ogni valore dice da dove viene. Codice iniziale.

<p align="center">
  <img src="https://img.shields.io/badge/License-GPL%203.0--or--later-blue.svg" alt="GPL 3.0 or later">
  <img src="https://img.shields.io/badge/Stack-Markdown%20%2B%20Python-blue.svg" alt="Markdown and Python">
  <img src="https://img.shields.io/badge/Languages-8-00E5FF.svg" alt="Languages">
  <img src="https://img.shields.io/badge/Tests-3-2EA043.svg" alt="Tests">
  <img src="https://img.shields.io/badge/Version-0.0.3-0f6cbd.svg" alt="Version">
  <img src="https://img.shields.io/badge/Maturity-scaffolding-FFB020.svg" alt="Maturity">
</p>

---

**Verifica di onestà - cosa funziona oggi:** una libreria Rust e una riga di comando che importano un elenco di parti da un file CSV in una base di dati JSON conservata come file accanto al progetto (`import`, `check-csv`, `list`) e che trovano ciò che non va in una base: identità duplicate, osservazioni senza fonte, fonti in disaccordo, schede segnate come verificate con qualcosa in sospeso e schede senza origine o licenza registrate; rifiutano di esportare dati senza licenza e non sostituiscono mai un file esistente senza un ordine esplicito. Tutto è provato solo su dati prodotti dai propri test e sui piccoli file di `examples/`. Non esistono: altri formati di importazione (solo il CSV di `docs/CSV_FORMAT.md`), l'unione di un'importazione con una base esistente e il collegamento con simboli e impronte dello strumento di progetto.

---

> **Stato di sicurezza funzionale e informatica:** questo repository è ai primi passi e non è un sistema di sicurezza funzionale. Nulla di quanto scritto qui sostituisce i limiti, gli interblocchi e l'arresto di emergenza della macchina e del suo controllore.

## 📋 In breve

| | |
|---|---|
| Versione | `0.0.3` |
| Maturità | Impalcatura |
| Test | 3 |
| Stack tecnologico | Markdown, Python 3.10+ |
| Su hardware reale | Non ancora eseguito su hardware |

**Vai a:** [Panoramica](#-panoramica) · [Architettura](#-architettura) · [Avvio rapido](#-avvio-rapido) · [Test](#-test) · [Struttura del repository](#-struttura-del-repository) · [Progetti correlati](#-progetti-correlati)

## 🚦 Stato del progetto

Ogni riga è una parte di questo repository e il semaforo dice a che punto è oggi. È scritta in base a ciò che il repository può mostrare (test, compilazioni, esecuzioni), non a ciò che è previsto.

🟢 Fatto e verificato · 🟡 Funziona, con limiti noti · 🟠 Fatto a metà o non ancora verificato · 🔴 Non fatto o non funzionante

| Area | Stato | Cosa significa qui |
|---|---|---|
| Controlli del repository | 🟡 | Il controllo del repository, quello delle immagini e i 3 test della base girano nella CI e passano. |
| Test | 🟡 | I 46 test del codice Rust e i 3 della base sono eseguiti dalla CI. Usano solo dati prodotti dai test stessi e i piccoli file di `examples/`. |
| Hardware reale | 🟠 | Non eseguito su nessuna macchina né scheda: nulla qui è stato provato su hardware. |
| Funzione pianificata | 🟠 | Scritto: la scheda con identità esatta, la provenienza di ogni osservazione, i conflitti, lo stato di fiducia, l'origine e la licenza, e l'importazione di un elenco di parti CSV in un file JSON. Non scritto: altri importatori, il collegamento con simboli e impronte e i datasheet. |
| Requisiti | 🟠 | I 6 requisiti sono a livello di prototipo: codice e test esistono, solo su dati prodotti dai test; nessuno è validato. |

## 🎯 Panoramica

* **Cosa esiste:** una libreria e una riga di comando, senza finestra, che leggono una base di dati JSON di componenti, la controllano (`check`), mostrano un componente con tutte le sue osservazioni (`show`), elencano i componenti (`list`) e la esportano solo se ogni scheda annota la licenza dei suoi dati (`export`); leggono anche un elenco di parti CSV in un nuovo file di base (`import`, con `check-csv` per controllare solo il file) e non sostituiscono mai un file esistente senza un ordine esplicito. **Pianificato:** altri importatori, il collegamento con simboli e impronte dello strumento di progetto e un tipo di attività del server.

## 🧩 Ruolo in EH3D

EH3D-COMPONENT-INTELLIGENCE fa parte della famiglia PHOENIX della EH3D Fabrication Suite. Cosa farà, cosa non farà ancora, i suoi requisiti e le sue relazioni con altri repository sono scritti nella cartella `docs/`.

## 🏛️ Architettura

<p align="center">
  <img src="images/architecture.svg" alt="Architettura di EH3D-COMPONENT-INTELLIGENCE: chi lo usa, che cosa contiene e da che cosa dipende" width="100%">
</p>

Come si incastrano le parti di questo repository: chi lo usa, che cosa contiene e da che cosa dipende. L'immagine è disegnata da `docs/architecture.json` e verificata rispetto al codice.

## 🚀 Avvio rapido

Serve Python 3.10 o successivo; non c'è nulla da installare.

```bash
python -I tools/ci_validate.py
python -I tools/make_architecture.py --check
python -I tools/make_structure.py --check --readmes
python -I -m unittest discover -s tools -p "test_*.py"
```

## 🧪 Test

La base ha 3 test (`python -I -m unittest discover -s tools -p "test_*.py"`), e passano. La CI esegue il controllo del repository, quello delle immagini e questi test.

## 🏗️ Compilazione e rilascio

La versione sta nel file `VERSION` e in `eh3d.project.json`, e ogni versione ha la sua voce in [CHANGELOG.md](CHANGELOG.md). Non c'è nulla da compilare; le versioni vengono pubblicate su GitHub.

## 📂 Struttura del repository

<p align="center">
  <img src="images/repository-structure.svg" alt="Struttura del repository EH3D-COMPONENT-INTELLIGENCE" width="100%">
</p>

<details>
<summary>Versione testuale dell'albero</summary>

```text
EH3D-COMPONENT-INTELLIGENCE/
├── docs/
│   ├── architecture.json structure.json         i dati delle due immagini
│   ├── SCOPE.md REQUIREMENTS.md                 ciò che è pianificato e i suoi requisiti
│   ├── DEPENDENCIES.md THREAT_MODEL.md          relazioni con altri repository e modello delle minacce
│   ├── DECISIONS.md STATUS.md                   decisioni applicabili qui e stato onesto
│   ├── NOT_YET.md RULES.md                      idee per dopo e regole per scrivere il codice
│   ├── CSV_FORMAT.md DATABASE_FORMAT.md         i formati di file: il CSV da importare e la base JSON
│   └── adr/                                     i registri delle decisioni accettate
├── AGENTS.md                                    regole per chi modifica il repository
├── src/                                         la libreria e la riga di comando
├── examples/                                    piccoli file CSV e un file di impostazioni per provare i comandi
├── Cargo.toml Cargo.lock deny.toml              pacchetto Rust e politica delle dipendenze
├── tools/                                       il controllo del repository, i suoi test e gli strumenti delle immagini
├── images/                                      banner e le immagini di Architettura e Struttura del repository
├── .github/                                     workflow di CI e della dashboard, modelli di issue e pull request
├── eh3d.project.json                            manifesto: versione, ruolo, maturità e comando di compilazione
├── VERSION                                      la versione del repository
├── CHANGELOG.md                                 cosa è cambiato in ogni versione
├── SECURITY.md SUPPORT.md                       come segnalare una vulnerabilità e dove chiedere
└── CONTRIBUTING.md CODE_OF_CONDUCT.md LICENSE   come contribuire, codice di condotta e licenza
```

</details>

## 🗺️ Roadmap e limiti noti

**Limiti noti oggi**

* Codice iniziale: provato solo su dati dei propri test e sui piccoli file di `examples/`; l'unico formato di importazione è l'elenco di parti CSV, un'importazione non si può unire a una base esistente e non c'è collegamento con lo strumento di progetto.

**Prossimi passi**

* Attendere che i contratti da cui dipende questo progetto siano accettati (vedi `docs/DEPENDENCIES.md`), poi scrivere la prima versione con test e sostituire le righe di stato con risultati reali.

## 🌍 Lingue

Il README e i testi esistono in inglese, spagnolo, francese, italiano, tedesco, cinese, giapponese e portoghese europeo.

## 🔗 Progetti correlati

* [EH3D-DOCS](https://github.com/JuanenRac/EH3D-DOCS): la mappa della suite, la sua architettura, le convenzioni e i controlli condivisi
* Relazioni pianificate, non ancora costruite (EH3D-COMMON, EH3D-PHOENIX-PCB, EH3D-DESIGN-VERIFIER, EH3D-THERMAL-LAB): vedi [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)

## 📚 Documentazione

* [Le convenzioni della suite](https://github.com/JuanenRac/EH3D-DOCS/blob/main/docs/CONVENTIONS.md)
* [Registro delle modifiche](CHANGELOG.md)
* [Ambito, requisiti, dipendenze, modello delle minacce e decisioni](docs/SCOPE.md)

## 🤝 Contribuire e comunità

* Prima di una pull request esegui `python -I tools/ci_validate.py` e i test della base: tutto deve passare. Vedi [CONTRIBUTING.md](CONTRIBUTING.md) e il [codice di condotta](CODE_OF_CONDUCT.md).
* Il README esiste in otto lingue: una modifica al testo si fa in tutti e otto i file, e le due immagini sono disegnate da `docs/architecture.json` e `docs/structure.json` con `tools/make_architecture.py` e `tools/make_structure.py`.
* Segnala una vulnerabilità in privato: [SECURITY.md](SECURITY.md). Domande, idee e segnalazioni: electrohobby3d@gmail.com ([SUPPORT.md](SUPPORT.md)).

## 👤 AUTORE

**JuanenRac (Electro Hobby 3D)** · electrohobby3d@gmail.com

## 📜 LICENZA

Software: GPL-3.0-or-later. Vedi [LICENSE](LICENSE). I progetti hardware della suite usano la CERN OHL-S.

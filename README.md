# ✨ IcoGen Premium

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://www.buymeacoffee.com/MichelBernasconi)

## About
IcoGen è un generatore di icone multi-formato scritto interamente in **Rust** e potenziato da una bellissima interfaccia grafica moderna e fluida basata su **Slint**. 

Ideale per sviluppatori di app (Android, iOS) e web designer, IcoGen ti permette di prendere un'immagine sorgente (o infinite immagini in batch) e generare automaticamente tutte le dimensioni necessarie per i tuoi progetti, in vari formati.

## Funzionalità
- **Generazione in Batch:** Carica più file contemporaneamente e gestiscili in modo asincrono per il massimo delle performance.
- **Profili Preimpostati:** Scegli tra profili pronti all'uso (Android, iOS, Favicon) o specifica le tue dimensioni personalizzate.
- **Rimozione Sfondo Intelligente:** Rimuove automaticamente gli sfondi monocromatici e imposta il canale alpha, perfetto per convertire vecchi loghi `.jpg` in pulitissimi `.png` o `.ico`.
- **Interfaccia Declarativa Slint:** Animazioni fluide, tema scuro moderno, basso utilizzo della CPU rispetto alle classiche app Electron.

## Come iniziare

### Requisiti
- [Rust](https://www.rust-lang.org/tools/install) (Cargo) installato sul sistema.

### Avvio
```bash
git clone https://github.com/MichelBernasconi/IcoGen.git
cd IcoGen
cargo run --release
```

## Licenza
Distribuito con licenza MIT. Vedi `LICENSE` per maggiori informazioni.

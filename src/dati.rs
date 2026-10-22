use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Tutto ciò che vogliamo ricordare tra un avvio e l'altro.
#[derive(Serialize, Deserialize, Default)]
pub struct Dati {
    pub cartella: Option<String>,
    /// chiave = percorso completo dell'epub
    pub libri: HashMap<String, DatiLibro>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DatiLibro {
    /// (indice capitolo, righe scrollate): dove eravamo arrivati
    pub segnalibro: Option<(usize, u16)>,
    pub note: Vec<Nota>,
}

#[derive(Serialize, Deserialize)]
pub struct Nota {
    pub capitolo: usize,
    pub riga: u16,
    pub testo: String,
}

fn file() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default().join("epub_reader.json")
}

impl Dati {
    // ponytail: file mancante o illeggibile = dati vuoti, errori di scrittura ignorati
    pub fn carica() -> Self {
        fs::read_to_string(file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn salva(&self) {
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = fs::write(file(), s);
        }
    }
}

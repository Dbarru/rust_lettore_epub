use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Clear, List, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

use crate::dati::{Dati, Nota};
use crate::r#struct::{Book, Libreria};

/// Le "schermate" dell'app. `Copy` ci permette di leggerla senza spostarla.
#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Cartella,   // si digita il percorso della libreria
    Libri,
    Capitoli,
    Lettura,
    NuovaNota,  // si digita il testo della nota
    Note,       // elenco delle note del libro
    LeggiNota,  // testo completo di una nota
    Note_Globali, // elenco delle note di tutti i libri
}

pub struct App {
    exit: bool,
    screen: Screen,
    dati: Dati,
    /// (percorso completo, nome file), ordinati per nome
    books: Vec<(PathBuf, String)>,
    books_state: ListState,
    /// il libro aperto: `None` finché non ne scegli uno
    book: Option<Book>,
    chapters_state: ListState,
    notes_state: ListState,
    /// indice del capitolo che si sta leggendo
    capitolo: usize,
    /// di quante righe è scrollato il testo del capitolo
    scroll: u16,
    /// scroll del testo di una nota lunga
    nota_scroll: u16,
    /// ricerca nelle liste libri/capitoli: testo cercato e se lo stiamo digitando
    filtro: String,
    cercando: bool,
    /// testo che l'utente sta digitando (percorso o nota)
    input: String,
    errore: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            exit: false,
            screen: Screen::Cartella,
            dati: Dati::carica(),
            books: Vec::new(),
            books_state: ListState::default(),
            book: None,
            chapters_state: ListState::default(),
            notes_state: ListState::default(),
            capitolo: 0,
            scroll: 0,
            nota_scroll: 0,
            filtro: String::new(),
            cercando: false,
            input: String::new(),
            errore: None,
        };
        // se ricordiamo una cartella ancora valida, saltiamo la domanda iniziale
        if let Some(c) = app.dati.cartella.clone().filter(|c| Path::new(c).is_dir()) {
            app.carica_libri(c);
            app.screen = Screen::Libri;
        }
        app
    }

    fn carica_libri(&mut self, percorso: String) {
        let mut libreria = Libreria { path: String::new(), libri: HashMap::new() };
        libreria._books_list(percorso);
        // ci servono sia il percorso (per aprire il libro) sia il nome (per mostrarlo)
        self.books = libreria.libri.into_iter().collect();
        self.books.sort_by(|a, b| a.1.cmp(&b.1));
        self.books_state = ListState::default().with_selected(Some(0));
    }

    /// Indici (nella lista completa) delle voci che passano il filtro, senza distinguere maiuscole.
    /// Le liste mostrano solo queste: la riga selezionata va tradotta con `visibili()[riga]`.
    fn visibili(&self) -> Vec<usize> {
        let filtro = self.filtro.to_lowercase();
        let nomi: Vec<&str> = match self.screen {
            Screen::Libri => self.books.iter().map(|(_, n)| n.as_str()).collect(),
            Screen::Capitoli => {
                self.book.iter().flat_map(|b| b.chapter_list.iter().map(String::as_str)).collect()
            }
            _ => Vec::new(),
        };
        (0..nomi.len()).filter(|&i| nomi[i].to_lowercase().contains(&filtro)).collect()
    }

    /// Dopo ogni modifica del filtro la selezione riparte dalla prima voce.
    fn riparti_dalla_prima(&mut self) {
        match self.screen {
            Screen::Libri => self.books_state.select(Some(0)),
            Screen::Capitoli => self.chapters_state.select(Some(0)),
            _ => {}
        }
    }

    /// Tasti mentre si digita la ricerca (su/giù continuano a muovere la lista).
    fn on_search(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => self.filtro.push(c),
            KeyCode::Backspace => {
                self.filtro.pop();
            }
            KeyCode::Enter => self.cercando = false, // il filtro resta, si naviga nei risultati
            KeyCode::Esc => {
                self.filtro.clear();
                self.cercando = false;
            }
            _ => return,
        }
        self.riparti_dalla_prima();
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => self.on_key(code),
            Event::Mouse(mouse) => self.on_mouse(mouse),
            _ => {}
        }
        Ok(())
    }

    /// Il mouse viene tradotto in tasti: così le regole restano in un posto solo.
    fn on_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.on_key(KeyCode::Down),
            MouseEventKind::ScrollUp => self.on_key(KeyCode::Up),
            MouseEventKind::Down(MouseButton::Right) => self.on_key(KeyCode::Esc),
            MouseEventKind::Down(MouseButton::Left) => {
                let visibili = self.visibili().len();
                let (state, len) = match (self.screen, &self.book) {
                    (Screen::Libri, _) => (&mut self.books_state, visibili),
                    (Screen::Capitoli, Some(_)) => (&mut self.chapters_state, visibili),
                    (Screen::Note, Some(b)) => {
                        let n = self.dati.libri.get(&b.path).map_or(0, |d| d.note.len());
                        (&mut self.notes_state, n)
                    }
                    _ => return,
                };
                // riga 0 = bordo superiore; `offset` = quante voci sono scrollate fuori
                let Some(riga) = (mouse.row as usize).checked_sub(1) else { return };
                let i = riga + state.offset();
                if i >= len {
                    return;
                }
                if state.selected() == Some(i) {
                    self.on_key(KeyCode::Enter); // secondo click sulla stessa voce = apri
                } else {
                    state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn on_key(&mut self, code: KeyCode) {
        // mentre si digita, 'q' è una lettera come le altre
        if matches!(self.screen, Screen::Cartella | Screen::NuovaNota) {
            return self.on_input(code);
        }
        if self.cercando && !matches!(code, KeyCode::Up | KeyCode::Down) {
            return self.on_search(code);
        }
        if code == KeyCode::Char('q') {
            self.salva_posizione();
            self.exit = true;
            return;
        }
        // i tasti dipendono da dove siamo
        match self.screen {
            Screen::Libri => match code {
                KeyCode::Down => self.books_state.select_next(),
                KeyCode::Up => self.books_state.select_previous(),
                KeyCode::Enter => self.open_book(),
                KeyCode::Char('/') => self.cercando = true,
                KeyCode::Esc if !self.filtro.is_empty() => {
                    self.filtro.clear();
                    self.riparti_dalla_prima();
                }
                KeyCode::Char('c') => {
                    self.input = self.dati.cartella.clone().unwrap_or_default();
                    self.errore = None;
                    self.screen = Screen::Cartella;
                }
                KeyCode::Char('l')=> {
                    self.screen = Screen::Note_Globali;
                    self.notes_state.select(Some(0));
                }
                _ => {}
            },
            Screen::Capitoli => match code {
                KeyCode::Down => self.chapters_state.select_next(),
                KeyCode::Up => self.chapters_state.select_previous(),
                KeyCode::Enter => self.open_chapter(),
                KeyCode::Char('/') => self.cercando = true,
                // prima Esc toglie il filtro, il successivo torna ai libri
                KeyCode::Esc if !self.filtro.is_empty() => {
                    self.filtro.clear();
                    self.riparti_dalla_prima();
                }
                KeyCode::Esc => self.screen = Screen::Libri,
                _ => {}
            },
            Screen::Lettura => match code {
                KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(10),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
                KeyCode::Char('n') => {
                    self.input.clear();
                    self.screen = Screen::NuovaNota;
                }
                KeyCode::Char('l') => {
                    self.notes_state = ListState::default().with_selected(Some(0));
                    self.screen = Screen::Note;
                }
                KeyCode::Esc => {
                    self.salva_posizione();
                    self.chapters_state.select(Some(self.capitolo));
                    self.screen = Screen::Capitoli;
                }
                _ => {}
            },
            Screen::Note => match code {
                KeyCode::Down => self.notes_state.select_next(),
                KeyCode::Up => self.notes_state.select_previous(),
                KeyCode::Enter => {
                    self.nota_scroll = 0;
                    self.screen = Screen::LeggiNota;
                }
                KeyCode::Char('g') => self.vai_alla_nota(),
                KeyCode::Char('d') | KeyCode::Delete => self.elimina_nota(),
                KeyCode::Esc => self.screen = Screen::Lettura,
                _ => {}
            },
            Screen::LeggiNota => match code {
                KeyCode::Down => self.nota_scroll = self.nota_scroll.saturating_add(1),
                KeyCode::Up => self.nota_scroll = self.nota_scroll.saturating_sub(1),
                KeyCode::Char('g') => self.vai_alla_nota(),
                KeyCode::Esc => self.screen = Screen::Note,
                _ => {}
            },

            Screen::Note_Globali => match code {
                KeyCode::Down => self.notes_state.select_next(),
                KeyCode::Up => self.notes_state.select_previous(),
                _ => {},
            },
            Screen::Cartella | Screen::NuovaNota => {}
        }
    }

    /// Tasti nelle schermate in cui si scrive.
    fn on_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => self.conferma_input(),
            KeyCode::Esc => {
                if self.screen == Screen::NuovaNota {
                    self.screen = Screen::Lettura;
                } else if self.dati.cartella.is_some() {
                    self.screen = Screen::Libri;
                } else {
                    self.exit = true; // prima volta e niente cartella: l'unica uscita
                }
            }
            _ => {}
        }
    }

    fn conferma_input(&mut self) {
        if self.screen == Screen::Cartella {
            // chi incolla da "Copia come percorso" si porta dietro le virgolette
            let percorso = self.input.trim().trim_matches('"').to_string();
            if !Path::new(&percorso).is_dir() {
                self.errore = Some("Cartella non trovata".to_string());
                return;
            }
            self.carica_libri(percorso.clone());
            self.dati.cartella = Some(percorso);
            self.dati.salva();
            self.errore = None;
            self.screen = Screen::Libri;
        } else {
            let testo = self.input.trim().to_string();
            if let (false, Some(book)) = (testo.is_empty(), &self.book) {
                let nota = Nota { capitolo: self.capitolo, riga: self.scroll, testo };
                self.dati.libri.entry(book.path.clone()).or_default().note.push(nota);
                self.dati.salva();
            }
            self.screen = Screen::Lettura;
        }
    }

    /// Ricorda a che punto siamo, se stavamo leggendo.
    fn salva_posizione(&mut self) {
        if self.screen != Screen::Lettura {
            return;
        }
        if let Some(book) = &self.book {
            self.dati.libri.entry(book.path.clone()).or_default().segnalibro =
                Some((self.capitolo, self.scroll));
            self.dati.salva();
        }
    }

    /// Enter nella lista dei libri: apre l'epub selezionato e passa ai capitoli.
    fn open_book(&mut self) {
        let Some(riga) = self.books_state.selected() else { return };
        let Some(&i) = self.visibili().get(riga) else { return };
        let Some((path, _)) = self.books.get(i) else { return };
        let path = path.to_string_lossy().into_owned();
        self.filtro.clear(); // la ricerca vale per una lista alla volta
        self.book = Some(Book::new_book(path.clone()));
        // partiamo dal capitolo del segnalibro, se c'è
        let inizio = self.dati.libri.get(&path).and_then(|d| d.segnalibro).map_or(0, |(c, _)| c);
        self.chapters_state = ListState::default().with_selected(Some(inizio));
        self.screen = Screen::Capitoli;
    }

    /// Enter nella lista dei capitoli: carica il testo e passa alla lettura.
    fn open_chapter(&mut self) {
        let Some(riga) = self.chapters_state.selected() else { return };
        let Some(&i) = self.visibili().get(riga) else { return };
        self.filtro.clear();
        if let Some(book) = self.book.as_mut() {
            book.chapter(i);
            self.capitolo = i;
            // se è il capitolo del segnalibro, riprendiamo dalla riga salvata
            self.scroll = match self.dati.libri.get(&book.path).and_then(|d| d.segnalibro) {
                Some((c, riga)) if c == i => riga,
                _ => 0,
            };
            self.screen = Screen::Lettura;
        }
    }

    /// Enter nell'elenco note: salta al capitolo e alla riga della nota.
    fn vai_alla_nota(&mut self) {
        let Some(i) = self.notes_state.selected() else { return };
        let Some(book) = self.book.as_mut() else { return };
        let Some(nota) = self.dati.libri.get(&book.path).and_then(|d| d.note.get(i)) else { return };
        let (capitolo, riga) = (nota.capitolo, nota.riga);
        book.chapter(capitolo);
        self.capitolo = capitolo;
        self.scroll = riga;
        self.screen = Screen::Lettura;
    }

    fn elimina_nota(&mut self) {
        let (Some(i), Some(book)) = (self.notes_state.selected(), &self.book) else { return };
        if let Some(d) = self.dati.libri.get_mut(&book.path) {
            if i < d.note.len() {
                d.note.remove(i);
                self.dati.salva();
            }
        }
    }

    /// Titolo della lista; se c'è una ricerca attiva la mostra (con cursore mentre si digita).
    fn titolo_ricerca(&self, base: &str) -> String {
        if self.cercando {
            format!(" {base} - cerca: {}█ ", self.filtro)
        } else if !self.filtro.is_empty() {
            format!(" {base} - filtro: {} (Esc per toglierlo) ", self.filtro)
        } else {
            format!(" {base} ")
        }
    }

    // allora devi implementare ancora questa funzione. ma ci sei quasi. ci sei quasi davvero. ti stai facendo aiutare da gemini notebook, ma molte cose in realtà ci eri gia arrivato 
    //quindi forse ci stai iniziando a capire qualcosa. non so quanto abbia senso imparare a programmare effettivamente, ma tutto fa brodo. prima o poi ti torna utile di sicuro 
    // dai che ce la fai.
    fn flatten_notes() -> Vec<>{

    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match self.screen {
            Screen::Cartella => {
                let mut righe = vec![Line::from(format!("{}█", self.input))];
                if let Some(e) = &self.errore {
                    righe.push(Line::from(e.as_str().red()));
                }
                let testo = Paragraph::new(righe).block(cornice(
                    " Percorso della cartella con gli epub ",
                    " Conferma <Invio> Esci/Annulla <Esc> ",
                    false,
                ));
                frame.render_widget(testo, area);
            }
            Screen::Libri => {
                let voci = self.visibili().iter().map(|&i| self.books[i].1.clone()).collect();
                let titolo = self.titolo_ricerca("Seleziona un libro");
                let list = lista(voci, &titolo, " Apri <Invio> Cerca </> Cartella <C> Note <l> ");
                frame.render_stateful_widget(list, area, &mut self.books_state);
            }
            Screen::Capitoli => {
                if let Some(book) = &self.book {
                    let segnalibro = self.dati.libri.get(&book.path).and_then(|d| d.segnalibro);
                    let voci = self
                        .visibili()
                        .iter()
                        .map(|&i| {
                            let marca = if segnalibro.map(|(s, _)| s) == Some(i) { "▶" } else { " " };
                            format!("{marca} {}", book.chapter_list[i])
                        })
                        .collect();
                    let titolo = self.titolo_ricerca("Seleziona un capitolo (▶ = dove eri rimasto)");
                    let list = lista(voci, &titolo, " Leggi <Invio> Cerca </> Indietro <Esc> ");
                    frame.render_stateful_widget(list, area, &mut self.chapters_state);
                }
            }
            Screen::Lettura | Screen::NuovaNota => {
                if let Some(book) = &self.book {
                    let note = self.dati.libri.get(&book.path).map_or(0, |d| {
                        d.note.iter().filter(|n| n.capitolo == self.capitolo).count()
                    });
                    let titolo = format!(" Lettura - capitolo {} ({note} note) ", self.capitolo + 1);
                    let testo = Paragraph::new(book.chapter.as_str())
                        .wrap(Wrap { trim: false })
                        .scroll((self.scroll, 0))
                        .block(cornice(
                            &titolo,
                            " Scorri <↑↓ PgUp PgDn> Nota <N> Elenco note <L> Indietro <Esc> ",
                            true,
                        ));
                    frame.render_widget(testo, area);
                }
                if self.screen == Screen::NuovaNota {
                    // finestra in basso, sopra al testo: cresce con la nota, fino a metà schermo
                    let campo = Paragraph::new(format!("{}█", self.input))
                        .wrap(Wrap { trim: false })
                        .block(cornice(" Nuova nota ", " Salva <Invio> Annulla <Esc> ", false));
                    // line_count tiene conto anche dei bordi della cornice
                    let righe = campo.line_count(area.width) as u16;
                    let h = righe.max(3).min(area.height / 2).max(3).min(area.height);
                    let r = Rect::new(area.x, area.bottom() - h, area.width, h);
                    // se supera metà schermo, mostriamo la fine (dove si sta scrivendo)
                    let campo = campo.scroll((righe.saturating_sub(h), 0));
                    frame.render_widget(Clear, r);
                    frame.render_widget(campo, r);
                }
            }
            Screen::Note => {
                if let Some(book) = &self.book {
                    let voci = self
                        .dati
                        .libri
                        .get(&book.path)
                        .map(|d| &d.note[..])
                        .unwrap_or_default()
                        .iter()
                        .map(|n| format!("cap. {} riga {}: {}", n.capitolo + 1, n.riga, n.testo))
                        .collect();
                    let list = lista(
                        voci,
                        " Note del libro ",
                        " Leggi <Invio> Vai al punto <G> Elimina <D> Indietro <Esc> ",
                    );
                    frame.render_stateful_widget(list, area, &mut self.notes_state);
                }
            }
            Screen::LeggiNota => {
                let nota = self.book.as_ref().and_then(|b| {
                    let i = self.notes_state.selected()?;
                    self.dati.libri.get(&b.path)?.note.get(i)
                });
                if let Some(n) = nota {
                    let titolo = format!(" Nota - capitolo {} riga {} ", n.capitolo + 1, n.riga);
                    let testo = Paragraph::new(n.testo.as_str())
                        .wrap(Wrap { trim: false })
                        .scroll((self.nota_scroll, 0))
                        .block(cornice(
                            &titolo,
                            " Scorri <↑↓> Vai al punto <G> Indietro <Esc> ",
                            true,
                        ));
                    frame.render_widget(testo, area);
                }
            }
            Screen:: Note_Globali =>{

            }
        }
    }
}

/// Cornice con titolo in alto e istruzioni in basso (+ "Quit <Q>" se `quit`).
fn cornice<'a>(titolo: &'a str, istruzioni: &'a str, quit: bool) -> Block<'a> {
    let mut parti = vec![istruzioni.into()];
    if quit {
        parti.extend([" Quit ".into(), "<Q> ".blue().bold()]);
    }
    Block::bordered()
        .title(Line::from(titolo.bold()).centered())
        .title_bottom(Line::from(parti).centered())
        .border_set(border::THICK)
}

/// Lista evidenziata: la usiamo per libri, capitoli e note.
fn lista<'a>(voci: Vec<String>, titolo: &'a str, istruzioni: &'a str) -> List<'a> {
    List::new(voci)
        .block(cornice(titolo, istruzioni, true))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ")
}

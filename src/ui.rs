use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

use crate::r#struct::{Book, Libreria};

/// Le tre "schermate" dell'app. `Copy` ci permette di leggerla senza spostarla.
#[derive(Clone, Copy)]
enum Screen {
    Libri,
    Capitoli,
    Lettura,
}

pub struct App {
    exit: bool,
    screen: Screen,
    /// (percorso completo, nome file), ordinati per nome
    books: Vec<(PathBuf, String)>,
    books_state: ListState,
    /// il libro aperto: `None` finché non ne scegli uno
    book: Option<Book>,
    chapters_state: ListState,
    /// di quante righe è scrollato il testo del capitolo
    scroll: u16,
}

impl App {
    pub fn new(percorso: String) -> Self {
        let mut libreria = Libreria { path: String::new(), libri: HashMap::new() };
        libreria._books_list(percorso);

        // ci servono sia il percorso (per aprire il libro) sia il nome (per mostrarlo)
        let mut books: Vec<(PathBuf, String)> = libreria.libri.into_iter().collect();
        books.sort_by(|a, b| a.1.cmp(&b.1));

        Self {
            exit: false,
            screen: Screen::Libri,
            books,
            books_state: ListState::default().with_selected(Some(0)),
            book: None,
            chapters_state: ListState::default(),
            scroll: 0,
        }
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
                let (state, len) = match (self.screen, &self.book) {
                    (Screen::Libri, _) => (&mut self.books_state, self.books.len()),
                    (Screen::Capitoli, Some(b)) => (&mut self.chapters_state, b.chapter_list.len()),
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
        if code == KeyCode::Char('q') {
            self.exit = true;
            return;
        }
        // i tasti dipendono da dove siamo
        match self.screen {
            Screen::Libri => match code {
                KeyCode::Down => self.books_state.select_next(),
                KeyCode::Up => self.books_state.select_previous(),
                KeyCode::Enter => self.open_book(),
                _ => {}
            },
            Screen::Capitoli => match code {
                KeyCode::Down => self.chapters_state.select_next(),
                KeyCode::Up => self.chapters_state.select_previous(),
                KeyCode::Enter => self.open_chapter(),
                KeyCode::Esc => self.screen = Screen::Libri,
                _ => {}
            },
            Screen::Lettura => match code {
                KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(10),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
                KeyCode::Esc => self.screen = Screen::Capitoli,
                _ => {}
            },
        }
    }

    /// Enter nella lista dei libri: apre l'epub selezionato e passa ai capitoli.
    fn open_book(&mut self) {
        let Some(i) = self.books_state.selected() else { return };
        let Some((path, _)) = self.books.get(i) else { return };
        self.book = Some(Book::new_book(path.to_string_lossy().into_owned()));
        self.chapters_state = ListState::default().with_selected(Some(0));
        self.screen = Screen::Capitoli;
    }

    /// Enter nella lista dei capitoli: carica il testo e passa alla lettura.
    fn open_chapter(&mut self) {
        let Some(i) = self.chapters_state.selected() else { return };
        if let Some(book) = self.book.as_mut() {
            book.chapter(i);
            self.scroll = 0;
            self.screen = Screen::Lettura;
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match self.screen {
            Screen::Libri => {
                let list = lista(
                    self.books.iter().map(|(_, nome)| nome.as_str()),
                    " Seleziona un libro ",
                    " Apri <Invio> ",
                );
                frame.render_stateful_widget(list, area, &mut self.books_state);
            }
            Screen::Capitoli => {
                if let Some(book) = &self.book {
                    let list = lista(
                        book.chapter_list.iter().map(String::as_str),
                        " Seleziona un capitolo ",
                        " Leggi <Invio> Indietro <Esc> ",
                    );
                    frame.render_stateful_widget(list, area, &mut self.chapters_state);
                }
            }
            Screen::Lettura => {
                if let Some(book) = &self.book {
                    let testo = Paragraph::new(book.chapter.as_str())
                        .wrap(Wrap { trim: false })
                        .scroll((self.scroll, 0))
                        .block(cornice(" Lettura ", " Scorri <↑↓ PgUp PgDn> Indietro <Esc> "));
                    frame.render_widget(testo, area);
                }
            }
        }
    }
}

/// Cornice con titolo in alto e istruzioni in basso (+ "Quit <Q>" sempre).
fn cornice<'a>(titolo: &'a str, istruzioni: &'a str) -> Block<'a> {
    Block::bordered()
        .title(Line::from(titolo.bold()).centered())
        .title_bottom(
            Line::from(vec![istruzioni.into(), " Quit ".into(), "<Q> ".blue().bold()]).centered(),
        )
        .border_set(border::THICK)
}

/// Lista evidenziata: la usiamo sia per i libri sia per i capitoli.
fn lista<'a>(
    voci: impl Iterator<Item = &'a str>,
    titolo: &'a str,
    istruzioni: &'a str,
) -> List<'a> {
    List::new(voci)
        .block(cornice(titolo, istruzioni))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ")
}

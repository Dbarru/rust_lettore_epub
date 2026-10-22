use std::collections::HashMap;
use std::io;
use crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, execute};
mod r#struct;
mod r#ui;
use r#struct::Libreria;
use r#struct::Book;
use r#ui::App;

// use roxmltree::Document;
// use scraper::{Html, Selector};
fn main()-> io::Result<()> {
    // let mut test_libreria = Libreria{
    //     path : String::new(),
    //     libri : HashMap::new(),
    // };
    // test_libreria._books_list( r"C:\Users\Burragato.MASCIA\Desktop\progetti\rust\epub".to_string());

    // let mut primo_libro = Book::new_book("C:\\Users\\Burragato.MASCIA\\Desktop\\progetti\\rust\\epub\\Vento e Verità Brandon Sanderson z-library.sk, 1lib.sk, z-lib.sk.epub".to_string());
    // primo_libro.chapter(40 as usize);
    // println!("{}", primo_libro.chapter );

    let percorso = r"C:\Users\Burragato.MASCIA\Desktop\progetti\rust\epub".to_string();
    ratatui::run(|terminal| {
        execute!(io::stdout(), EnableMouseCapture)?;
        let esito = App::new(percorso).run(terminal);
        execute!(io::stdout(), DisableMouseCapture)?; // ripristina il terminale
        esito
    })
}
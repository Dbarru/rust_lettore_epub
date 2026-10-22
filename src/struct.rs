use std::collections::HashMap;
use std::io::{Read, Seek};
use std::fs::File;
use crate::io::Error;
use scan_dir::ScanDir;
use std::path::PathBuf;
use zip::ZipArchive;
use funzioni::{read_container_xml, read_content_xml, 
    extract_manifest, extract_spine, build_reading_order, read_text};


// we used this struct to manage the boocks.
pub struct Book{
   pub path : String, 
   pub chapter_list : Vec<String>,
   pub chapter_path : HashMap<String, String>,
   pub chapter_folder : String,
   pub chapter: String,
   pub book : ZipArchive<File>,
}
impl Book{
    pub fn new_book(path1 : String) -> Self {
        let file = File::open(&path1).expect("failed to open the file");
        let mut book = ZipArchive::new(file).expect("non sono riuscito a estrarre il file zip");
        let content = read_container_xml(&mut book).expect("non sono riuscito a trovare il container");
        let indice = read_content_xml(&mut book, &content).expect("non ho trovato l'indice");
        let parse1 = roxmltree::Document::parse(&indice).unwrap();
        let chapter_path = extract_manifest(&parse1);
        let capitoli_sparsi = extract_spine(&parse1);
        let chapter_folder = match content.rsplit_once('/'){
            Some((cartella, _)) => cartella.to_string(),
            None => String::new(),
        };
        let chapter_list = build_reading_order(&capitoli_sparsi, &chapter_path, &chapter_folder);
        Book {
            path: path1,
            chapter_list,
            chapter_path,
            chapter_folder,
            chapter: String::new(),
            book,
        }
    }

    // ti manca di fare la funzine per leggere i capitoli 
    pub fn chapter(&mut self, capitolo_selezionato: usize){
        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let chapt_xml = read_content_xml(&mut self.book, &self.chapter_list.get(capitolo_selezionato).expect("problemi con il capitolo selezonato")).expect("problemi con l'xml del capitolo");
        let cahpt_parsato = roxmltree::Document::parse_with_options(&chapt_xml, opt).expect("parsing fallito");
        self.chapter = read_text(&cahpt_parsato);
    }
}


pub struct Libreria {
    pub path: String,
    pub libri : HashMap<PathBuf, String>,
}
impl Libreria{
    pub fn _books_list(&mut self, percorso_libreria: String){
        self.path = percorso_libreria;
        ScanDir::files().read(&self.path, |iter| {
            for (path, name) in iter {
                self.libri.insert(
                    path.path(),
                    name,
                );  
            }
        }).expect("Cartella non leggibile"); 
    }
}
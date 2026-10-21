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
struct Book<R,E> {
   path : File, 
   chapter_list : Vec<String>,
   chapter_path : HashMap<String, String>,
   chapter_folder : String,
   chapter: String,
   book : ZipArchive<File>,
}
impl Book{
    fn new_book(&mut self, path1 : String) {
        let mut capitoli_sparsi:Vec<String> = Vec::new();
        self.path = File::open(path1).expect("Failed to open file");
        self.book = zip::ZipArchive::new(&self.path).expect("...");
        let content = read_container_xml(&self.book).expect("non  ho trovato nulla");
        let indice = read_content_xml(&self.book, &content).expect("non ho trovato i contenuti");
        let parse1 = roxmltree::Document::parse(&indice).unwrap();
        self.chapter_path = extract_manifest(&parse1);
        capitoli_sparsi = extract_spine(&parse1);
        self.chapter_folder  = match content.rsplit_once('/'){
            Some((cartella, _)) => cartella.to_string(),
            None => String::new(),
        };
        self.chapter_list = build_reading_order(&capitoli_sparsi, &self.chapter_path, &self.chapter_folder);
    }


    // ti manca di fare la funzine per leggere i capitoli 
    fn chapter<R:Read + Seek + std::fmt::Debug>(&mut self, capitolo_selezionato: usize){
        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let chapt_xml = read_content_xml(&self.book, &self.chapter_list.get(capitolo_selezionato).expect("problemi con il capitolo selezonato")).expect("problemi con l'xml del capitolo");
        let cahpt_parsato = roxmltree::Document::parse_with_options(&chapt_xml, opt).expect("parsing fallito");
        self.chapter = read_text(&cahpt_parsato);
    }
}


struct libreria {
    path: String,
    libri : HashMap<PathBuf, String>
}
impl libreria{
    fn _books_list(mut self, percorso_libreria: String){
        self.path = percorso_libreria;
        ScanDir::files().read(self.path, |iter| {
            for (path, name) in iter {
                self.libri.insert(
                    path.path(),
                    name.to_string()
                );  
            }
        }); 
    }
}
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::fs::File;
use crate::io::Error;
use funzioni::{read_container_xml, read_content_xml, 
    extract_manifest, extract_spine, build_reading_order};

struct Book {
   path : File, 
   chapter_list : Vec<String>,
   chapter_path : HashMap<String, String>,
   chapter_folder : String,
   chapter: String,
}
impl Book{
    fn new_book(&mut self, path1 : String) {
        let mut capitoli_sparsi:Vec<String> = Vec::new();
        self.path = File::open(path1).expect("Failed to open file");
        let mut zip = zip::ZipArchive::new(&self.path).expect("...");
        let content = read_container_xml(&mut zip).expect("non  ho trovato nulla");
        let indice = read_content_xml(&mut zip, &content).expect("non ho trovato i contenuti");
        let parse1 = roxmltree::Document::parse(&indice).unwrap();
        self.chapter_path = extract_manifest(&parse1);
        capitoli_sparsi = extract_spine(&parse1);
        self.chapter_folder  = match content.rsplit_once('/'){
            Some((cartella, _)) => cartella.to_string(),
            None => String::new(),
        };
        self.chapter_list = build_reading_order(&capitoli_sparsi, &self.chapter_path, &self.chapter_folder)
    }
}
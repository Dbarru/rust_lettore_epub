use std::io::{Read, Seek};
use std::fs::File;
use std::collections::HashMap;
use std::io;
use scan_dir::ScanDir;
mod r#struct;

// use roxmltree::Document;
// use scraper::{Html, Selector};
fn main() { 
    let file = File::open(r"C:\Users\Burragato.MASCIA\Desktop\progetti\rust\epub\Vento e Verità Brandon Sanderson z-library.sk, 1lib.sk, z-lib.sk.epub ")
        .expect("Failed to open file");

    let mut zip = zip::ZipArchive::new(file).expect("...");
    let content = read_container_xml(&mut zip).expect("non  ho trovato nulla");
    let indice = read_content_xml(&mut zip, &content).expect("non ho trovato i contenuti");
    let parse1 = roxmltree::Document::parse(&indice).unwrap();
    let manifest = extract_manifest(&parse1);
    let spine = extract_spine(&parse1);

    let cartella_capitoli: &str = match content.rsplit_once('/'){
        Some((cartella, _)) => cartella,
        None => "",
    };
    
    let ordine_libro1 = build_reading_order(&spine, &manifest, &cartella_capitoli);
    chapter_selection(&ordine_libro1, &mut zip);
}

fn read_container_xml<R:Read + Seek>(zip:&mut zip::ZipArchive<R>) -> zip::result::ZipResult<String> {
    let mut file1 = zip.by_name("META-INF/container.xml")?;
    let mut contenuto = String::new();
    file1.read_to_string(&mut contenuto)?;
    let cont1 = roxmltree::Document::parse(&contenuto).unwrap();
    
    let contenuto_parsato = cont1.descendants()
        .find(|n| n.has_tag_name("rootfile")).unwrap()
        .attribute("full-path").unwrap();
    Ok(contenuto_parsato.to_string())
}

fn read_content_xml<R:Read + Seek>(zip:&mut zip::ZipArchive<R>, path : &String) -> zip::result::ZipResult<String>{
    let mut file1 = zip.by_name(path)?;
    let mut contenuto = String::new();
    file1.read_to_string(&mut contenuto)?;
    Ok(contenuto)
}

fn extract_manifest(contenuto:&roxmltree::Document)-> HashMap<String, String>{
    let mut hashmap_indice = HashMap::new();
    for node in contenuto.descendants(){
        if node.has_tag_name("item"){
            let id:&str = node.attribute("id").unwrap();
            let href:&str = node.attribute("href").unwrap();
            hashmap_indice.insert(
                 id.to_string(),
                href.to_string()
            );
        }  
    }; 
    hashmap_indice
}

fn extract_spine(contenuto:&roxmltree::Document)-> Vec<String>{
    let mut spine_indice = Vec::new();
    for node in contenuto.descendants(){
        if node.has_tag_name("itemref"){
            let idref:&str = node.attribute("idref").unwrap();
            spine_indice.push(idref.to_string());
        }  
    }; 
    spine_indice
}

fn build_reading_order(vettore: &Vec<String>, mappa : &HashMap<String, String>, cartella_capitoli : &str) -> Vec<String>{
    let mut ordine_libro: Vec<String> = Vec::new();
    for ind in vettore {
        if cartella_capitoli.is_empty(){
            let percorso_completo: String = ind.to_string();
            ordine_libro.push(mappa.get(&percorso_completo).unwrap().to_string());
        }else{
            let esempio: String = ind.to_string();
            let  percorso_completo = format!("{}/{}", cartella_capitoli, mappa.get(&esempio).unwrap().to_string());
            ordine_libro.push(percorso_completo);
        }
        
    }
    ordine_libro
}

fn read_text(contenuto:&roxmltree::Document) -> String{
    let mut testo_completo = String::new();
    for paragrafo in contenuto.descendants(){
        if paragrafo.has_tag_name("p"){
            for nodo in paragrafo.descendants(){
                if nodo.is_text(){
                    let testo_parziale = nodo.text().unwrap();
                    testo_completo.push_str(testo_parziale);
                }
            }   
              testo_completo.push_str("\n") 
        }
    }
    testo_completo
}

fn chapter_selection<R:Read + Seek + std::fmt::Debug>(vettore_capitoli: &Vec<String>, zip:&mut zip::ZipArchive<R>) -> String{
    let num_capitoli : usize = vettore_capitoli.len();

    loop{
        println!("scegli un capitolo da 1 a {}", num_capitoli);
        let mut capitolo: String = String::new();
        io::stdin()
            .read_line(&mut capitolo)
            .expect("Failed to read line");

        let capitolo:usize =  match capitolo.trim().parse(){
            Ok(num) => num,
            Err(_) => continue,
        };
        
        let esempio: &String = vettore_capitoli.get(capitolo).expect("messaggio");

        
        let test: String = read_content_xml(zip, &esempio).expect("non ce la faccio piu");

        let opt = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };

        let testo_parsato = roxmltree::Document::parse_with_options(&test, opt).expect("parsing fallito");   
        let capitolo_testo = read_text(&testo_parsato); 
        println!("{}",capitolo_testo);
    }
}

fn _books_list(){
    ScanDir::files().read(r"C:\Users\Burragato.MASCIA\Desktop\progetti\rust\epub", |iter| {
        for (path, name) in iter {
            println!("File {:?} in {:?}", name, path);
        }
    }).unwrap()  
}
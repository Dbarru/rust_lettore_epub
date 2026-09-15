use std::io::{Read, Seek};
use std::fs::File;
use std::collections::HashMap;
// use roxmltree::Document;


fn main() { 
    let file = File::open(r"C:\Users\Burragato.MASCIA\Desktop\progetti\rust\epub\Vento e Verità Brandon Sanderson z-library.sk, 1lib.sk, z-lib.sk.epub")
        .expect("Failed to open file");

    let mut zip = zip::ZipArchive::new(file).expect("...");
    let content = read_container_xml(&mut zip).expect("non  ho trovato nulla");
    let indice = read_content_xml(&mut zip, &content).expect("non ho trovato i contenuti");
    let parse1 = roxmltree::Document::parse(&indice).unwrap();
    let manifest = extract_manifest(&parse1);
    let spine = extract_spine(&parse1);
    let ordine_libro1 = build_reading_order(&spine, &manifest);
    println!("{:?}",ordine_libro1 )
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
    // println!("{contenuto}");
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

fn build_reading_order(vettore: &Vec<String>, mappa : &HashMap<String, String>) -> Vec<String>{
    let mut ordine_libro: Vec<String> = Vec::new();
    for ind in vettore {
        ordine_libro.push(mappa.get(ind).unwrap().to_string());
    }
    ordine_libro
}
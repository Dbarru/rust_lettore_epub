use std::collections::HashMap;
use std::io::{Read, Seek};
use std::fs::File;

pub fn read_container_xml<R:Read + Seek>(zip:&mut zip::ZipArchive<R>) -> zip::result::ZipResult<String> {
    let mut file1 = zip.by_name("META-INF/container.xml")?;
    let mut contenuto = String::new();
    file1.read_to_string(&mut contenuto)?;
    let cont1 = roxmltree::Document::parse(&contenuto).unwrap();
    
    let contenuto_parsato = cont1.descendants()
        .find(|n| n.has_tag_name("rootfile")).unwrap()
        .attribute("full-path").unwrap();
    Ok(contenuto_parsato.to_string())
}

pub fn read_content_xml<R:Read + Seek>(zip:&mut zip::ZipArchive<R>, path : &String) -> zip::result::ZipResult<String>{
    let mut file1 = zip.by_name(path)?;
    let mut contenuto = String::new();
    file1.read_to_string(&mut contenuto)?;
    Ok(contenuto)
}

pub fn extract_manifest(contenuto:&roxmltree::Document)-> HashMap<String, String>{
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

pub fn extract_spine(contenuto:&roxmltree::Document)-> Vec<String>{
    let mut spine_indice = Vec::new();
    for node in contenuto.descendants(){
        if node.has_tag_name("itemref"){
            let idref:&str = node.attribute("idref").unwrap();
            spine_indice.push(idref.to_string());
        }  
    }; 
    spine_indice
}

pub fn build_reading_order(vettore: &Vec<String>, mappa : &HashMap<String, String>, cartella_capitoli : &str) -> Vec<String>{
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

pub fn read_text(contenuto:&roxmltree::Document) -> String{
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
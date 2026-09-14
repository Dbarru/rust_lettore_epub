use std::io::{Read, Seek};
use std::fs::File;
// use roxmltree::Document;


fn main() { 
    let file = File::open(r"C:\Users\Burragato.MASCIA\Desktop\progetti\rust\epub\Vento e Verità Brandon Sanderson z-library.sk, 1lib.sk, z-lib.sk.epub")
        .expect("Failed to open file");

    let mut zip = zip::ZipArchive::new(file).expect("...");
    let content = read_container_xml(&mut zip).expect("non  ho trovato nulla");
    read_content_xml(&mut zip, &content).expect("non ho trovato i contenuti");
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

fn read_content_xml<R:Read + Seek>(zip:&mut zip::ZipArchive<R>, path : &String) -> zip::result::ZipResult<()>{
    let mut file1 = zip.by_name(path)?;
    let mut contenuto = String::new();
    file1.read_to_string(&mut contenuto)?;
    println!("{contenuto}");
    Ok(())
}

// fn list_zip_contents(reader: impl Read + Seek) -> zip::result::ZipResult<()> {
//     let mut zip = zip::ZipArchive::new(reader)?;

//     for i in 0..zip.len() {
//         let file = zip.by_index(i)?;
//         println!("Filename: {}", file.name());
//     }

//     Ok(())
// }
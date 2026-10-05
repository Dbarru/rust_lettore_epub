// enum epub {
//     html,
//     xhtml
// }
// impl parse {
//     fn parse(&self){
//         match tipo {
//             epub::html =>
//             epub::xhtml => read_text(&self)
//         }
//     }
// }


// fn read_text(contenuto:&roxmltree::Document) -> String{
//     let mut testo_completo = String::new();
//     for paragrafo in contenuto.descendants(){
//         if paragrafo.has_tag_name("p"){
//             for nodo in paragrafo.descendants(){
//                 if nodo.is_text(){
//                     let testo_parziale = nodo.text().unwrap();
//                     testo_completo.push_str(testo_parziale);
//                 }
//             }   
//               testo_completo.push_str("\n") 
//         }
//     }
//     testo_completo
// }
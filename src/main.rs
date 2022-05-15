use std::env;
use std::ffi::OsStr;
use std::path::Path;

fn main() {
    let formats_entrants_acceptes = ["cbz", "cb7"];
    let formats_sortants_acceptes = ["cbz", "cb7"];

    // Bon, la ligne de commande ressemble à "nomDuProgramme NomDuFichier formatDuFichier"
    // TODO Vérifier le nombre d'arguments -> sinon afficher : Usage : "nomDuProgramme NomDuFichier formatDuFichier"
    let args: Vec<String> = env::args().collect();
    let nom_fichier_source = &args[1];
    let extension_fichier_sortant = args[2].clone();

    // VÉRIFICATIONS :
    // Le fichier existe-t-il ?
    let fichier_existant : bool = fichier_existe(nom_fichier_source);
    println!("Le fichier existe-t-il : {}", fichier_existant);
    // L'extension du fichier est-elle acceptée ?
    let extension_fichier_entrant : String = extension_fichier(nom_fichier_source);
    println!("L'extension trouvée : {:?}", extension_fichier_entrant);
    let extension_entrante_valide = extension_acceptee(formats_entrants_acceptes, extension_fichier_entrant);
    println!("L'extension entrante est valide : {}", extension_entrante_valide);
    // L'extension du fichier de sortie est-elle valide ?
    let extension_sortante_valide = extension_acceptee(formats_sortants_acceptes, extension_fichier_sortant);
    println!("L'extension sortante est valide : {}", extension_sortante_valide);

}

fn extension_acceptee(formats_acceptes: [&str; 2], extension_fichier: String) ->bool{
    return formats_acceptes.contains(&&**&extension_fichier);
}

fn extension_fichier(nom_fichier: &str) -> String {
    return String::from(Path::new(nom_fichier).extension().and_then(OsStr::to_str).as_deref().unwrap_or("extension inconnue"));
}

fn fichier_existe(nom_fichier_source: &String)->bool {
    return Path::new(nom_fichier_source).exists()
}
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use rand::Rng;


extern crate unrar;
use unrar::Archive;
use walkdir::{DirEntry, WalkDir};
use image::*;
use unrar::archive::Entry;
use webp::{Encoder, WebPMemory};
use zip::result::ZipError;
use zip::write::FileOptions;

extern crate zip;
extern crate walkdir;

const METHOD_STORED : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);

#[cfg(feature = "deflate")]
const METHOD_DEFLATED : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
#[cfg(not(feature = "deflate"))]
const METHOD_DEFLATED : Option<zip::CompressionMethod> = None;

#[cfg(feature = "bzip2")]
const METHOD_BZIP2 : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);
#[cfg(not(feature = "bzip2"))]
const METHOD_BZIP2 : Option<zip::CompressionMethod> = None;



fn main() {
    let formats_entrants_acceptes = ["cb7", "cbr", "cbz"];
    let formats_sortants_acceptes = ["cb7", "cbz"];

    // Bon, la ligne de commande ressemble à "nomDuProgramme NomDuFichier formatDuFichierSortant formatDesImages compression"
    // TODO Vérifier le nombre d'arguments -> sinon afficher : Usage : "nomDuProgramme NomDuFichier formatDuFichier"
    let args: Vec<String> = env::args().collect();
    let fichier_source: &Path = Path::new(&args[1]);

    let extension_fichier_sortant:String = args[2].clone();
    let format_des_images:String = args[3].clone();
    let compression_des_images:String = args[4].clone();

    // EXTRACTION DE L'ARCHIVE
    let fichier_source_extension:String = extension_fichier(&fichier_source);
    let fichier_source_nom:String = nom_fichier(&fichier_source);
    let fichier_source_chemin_absolu:String = chemin_absolu_fichier(&fichier_source);
    let fichier_source_dossier:String = dossier_fichier(&fichier_source);
    println!("extension fichier source : {}", fichier_source_extension);
    println!("nom fichier source : {}", fichier_source_nom);
    println!("dossier fichier source : {}", fichier_source_dossier);
    println!("chemin complet fichier source : {}", fichier_source_chemin_absolu);
    if !fichier_source.exists() { println!("Le fichier n'existe pas"); return;}
    if !formats_entrants_acceptes.contains(&&**&fichier_source_extension){ println!("Extension entrante non autorisée"); return; }
    println!("le fichier peut être décompressé");
    match fichier_source_extension.as_str() {
         "cb7" => println!("cb7"),
         "cbr" => compresser_images(extraire_archive_rar(&fichier_source_chemin_absolu, &fichier_source_dossier)),
         "cbz" => println!("cbz"),
         _ => println!("Extension entrante inconnue... encore ?"),
    }
}

fn compresser_images(donnees_extraction: (String, Vec<Entry>)) {
    println!("chemin complet fichier source : {:?}", donnees_extraction);
}


fn extraire_archive_rar(archive: &String, dossier_sortie: &String) -> (String, Vec<Entry>) {
    let chaine_aleatoire = chaine_aleatoire(10);
    return (
        String::from(Path::new(dossier_sortie).join(&chaine_aleatoire).to_str().unwrap()),
        Archive::new(archive.to_string()).extract_to(String::from(Path::new(dossier_sortie).join(&chaine_aleatoire).to_str().unwrap()).parse().unwrap()).unwrap().process().unwrap()
    );
}

fn chaine_aleatoire(taille: i32) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();

    let password: String = (0..taille)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    return password;
}

fn chemin_absolu_fichier(fichier: &Path) -> String {
    return String::from(fichier.clone().canonicalize().unwrap().to_str().unwrap());
}

fn extension_fichier(fichier: &Path) -> String {
    return String::from(fichier.clone().extension().and_then(OsStr::to_str).unwrap_or("extension entrante non reconnue"));
}

fn nom_fichier(fichier: &Path) -> String {
    return String::from(fichier.clone().file_stem().and_then(OsStr::to_str).unwrap_or("nom fichier entrant non reconnue"));
}

fn dossier_fichier(fichier: &Path) -> String {
    return String::from(fichier.parent().unwrap().to_str().unwrap_or("dossier parent introuvable"));
}

//
//     // VÉRIFICATIONS : // TODO Mettre un message d'erreur différent à chaque vérification.
//     // Le fichier existe-t-il ?
//     let fichier_existant : bool = fichier_existe(nom_fichier_source);
//     println!("Le fichier existe-t-il : {}", fichier_existant);
//     // L'extension du fichier est-elle acceptée ?
//     let extension_fichier_entrant : String = extension_fichier(nom_fichier_source);
//     println!("L'extension trouvée : {:?}", extension_fichier_entrant);
//     let extension_entrante_valide = extension_acceptee(formats_entrants_acceptes, &extension_fichier_entrant);
//     println!("L'extension entrante est valide : {}", extension_entrante_valide);
//     // L'extension du fichier de sortie est-elle valide ?
//     let extension_sortante_valide = extension_acceptee(formats_sortants_acceptes, &extension_fichier_sortant);
//     println!("L'extension sortante est valide : {}", extension_sortante_valide);
//     if extension_sortante_valide && extension_entrante_valide && fichier_existant {
//         println!("on lance !");
//         // C'est parti : -> on extraie les fichiers -> on compresse les fichiers -> on supprime les fichiers-> c'est tout
//
//         // 1° EXTRACTION
//         let fichier_test = "/home/sacha/Projets/rs_comic_shrinker/comics/test_cbr.cbr";
//         extraire_archive(extension_fichier_entrant, &fichier_test);
//         // 1A° COMPRESSION DES IMAGES -> webp
//         convertir_en_webp();
//         // 2° COMPRESSION
//         // let src_dir = "/home/sacha/Projets/rs_comic_shrinker/comics/extracted/berserk_test";
//         // let dst_file = "/home/sacha/Projets/rs_comic_shrinker/comics/test.cbz";
//         // for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2].iter() {
//         //     if method.is_none() { continue }
//         //     match doit(src_dir, dst_file, method.unwrap()) {
//         //         Ok(_) => println!("done: {} written to {}", src_dir, dst_file),
//         //         Err(e) => println!("Error: {:?}", e),
//         //     }
//         // }
//     }
// }
//
// fn convertir_en_webp(){
//     // Using `image` crate, open the included .jpg file
//     let img = image::open("/home/sacha/Projets/rs_comic_shrinker/comics/extracted/Berserk - Volume 01/Berserk - 00 p006-07 copy.jpg").unwrap();
//     let (w,h) = img.dimensions();
//     // Optionally, resize the existing photo and convert back into DynamicImage
//     let size_factor = 1.0;
//     let img: DynamicImage = image::DynamicImage::ImageRgba8(imageops::resize(
//         &img,
//         (w as f64 * size_factor) as u32,
//         (h as f64 * size_factor) as u32,
//         imageops::FilterType::Triangle,
//     ));
//
//     // Create the WebP encoder for the above image
//     let encoder: Encoder = Encoder::from_image(&img).unwrap();
//     // Encode the image at a specified quality 0-100
//     let webp: WebPMemory = encoder.encode(5f32);
//     // Define and write the WebP-encoded file to a given path
//     let output_path = Path::new("/home/sacha/Projets/rs_comic_shrinker/comics/extracted/Berserk - Volume 01").join("Berserk - 00 p006-07 copy").with_extension("webp");
//     std::fs::write(&output_path, &*webp).unwrap();
//     println!("image compressée");
// }
//
// fn doit(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
//     if !Path::new(src_dir).is_dir() {
//         return Err(ZipError::FileNotFound);
//     }
//
//     let path = Path::new(dst_file);
//     let file = File::create(&path).unwrap();
//
//     let walkdir = WalkDir::new(src_dir.to_string());
//     let it = walkdir.into_iter();
//
//     zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;
//
//     Ok(())
// }
//
// fn zip_dir<T>(it: &mut dyn Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod)
//               -> zip::result::ZipResult<()>
//     where T: Write+Seek
// {
//     let mut zip = zip::ZipWriter::new(writer);
//     let options = FileOptions::default()
//         .compression_method(method)
//         .unix_permissions(0o755);
//
//     let mut buffer = Vec::new();
//     for entry in it {
//         let path = entry.path();
//         let name = path.strip_prefix(Path::new(prefix)).unwrap();
//
//         // Write file or directory explicitly
//         // Some unzip tools unzip files with directory paths correctly, some do not!
//         if path.is_file() {
//             println!("adding file {:?} as {:?} ...", path, name);
//             zip.start_file_from_path(name, options)?;
//             let mut f = File::open(path)?;
//
//             f.read_to_end(&mut buffer)?;
//             zip.write_all(&*buffer)?;
//             buffer.clear();
//         } else if name.as_os_str().len() != 0 {
//             // Only if not root! Avoids path spec / warning
//             // and mapname conversion failed error on unzip
//             println!("adding dir {:?} as {:?} ...", path, name);
//             zip.add_directory_from_path(name, options)?;
//         }
//     }
//     zip.finish()?;
//     Result::Ok(())
// }
//
//


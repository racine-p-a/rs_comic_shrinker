use std::{env, fs};
use std::borrow::Borrow;
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
extern crate core;

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
    env::set_var("RUST_BACKTRACE", "full");
    let formats_entrants_acceptes = ["cb7", "cbr", "cbz"];
    let formats_sortants_acceptes = ["cb7", "cbz"];

    // Bon, la ligne de commande ressemble à "nomDuProgramme NomDuFichier formatDuFichierSortant formatDesImages compression"
    // TODO Vérifier le nombre d'arguments -> sinon afficher : Usage : "nomDuProgramme NomDuFichier formatDuFichier"
    let args: Vec<String> = env::args().collect();
    let fichier_source: &Path = Path::new(&args[1]);

    let extension_fichier_sortant:String = args[2].clone();
    let format_des_images:String = args[3].clone();
    let compression_des_images:f32 = args[4].clone().parse::<f32>().unwrap();

    // EXTRACTION DE L'ARCHIVE
    let fichier_source_extension:String = extension_fichier(&fichier_source);
    let fichier_source_nom:String = nom_fichier(&fichier_source);
    let fichier_source_chemin_absolu:String = chemin_absolu_fichier(&fichier_source);
    let fichier_source_dossier:String = dossier_fichier(&fichier_source);
    if !fichier_source.exists() { println!("Le fichier n'existe pas"); return;}
    if !formats_entrants_acceptes.contains(&&**&fichier_source_extension){ println!("Extension entrante non autorisée"); return; }
    match fichier_source_extension.as_str() {
         "cb7" => println!("cb7"),
         "cbr" => compresser_dossier(&extension_fichier_sortant, &compresser_images(&format_des_images, &compression_des_images, extraire_archive_rar(&fichier_source_chemin_absolu, &fichier_source_dossier), &fichier_source_nom)),
         "cbz" => println!("cbz"),
         _ => println!("Extension entrante inconnue... encore ?"),
    }
}

fn compresser_dossier(extension_fichier_sortant: &String, dossier_a_compresser: &PathBuf) {
    for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2].iter() {
        if method.is_none() { continue }
        match doit(dossier_a_compresser.to_str().unwrap(), dossier_a_compresser.with_extension(extension_fichier_sortant).to_str().unwrap(), method.unwrap()) {
            Ok(_) => println!("done: {} written to {}", dossier_a_compresser.to_str().unwrap(), dossier_a_compresser.with_extension(extension_fichier_sortant).to_str().unwrap()),
            Err(e) => println!("Error: {:?}", e),
        }
    }
    // todo supprimer les deux dossiers temporaires
}

fn doit(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }
    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();
    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir.into_iter();
    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;
    Ok(())
}

fn zip_dir<T>(it: &mut dyn Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod)
              -> zip::result::ZipResult<()>
    where T: Write+Seek
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();
        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {:?} as {:?} ...", path, name);
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}


fn compresser_images(format_des_images: &String, compression_des_images: &f32, donnees_extraction: (String, Vec<Entry>), nom_archive: &String) -> PathBuf {
    //println!("donnees extraction : {:?}", donnees_extraction);
    let data = match format_des_images.as_str() {
        "webp" => convertir_en_webp(&donnees_extraction.0, &compression_des_images, &nom_archive),
        _ => panic!("Erreur d'ouverture du fichier : "),
    };
    return data;
}

fn convertir_en_webp(dossier_parent:&String, compression: &&f32, nom_archive: &String) -> PathBuf {
    // Créons le dossier cible
    let dossier_cible = Path::new(&dossier_parent).parent().unwrap().join(nom_archive);

    for entry in WalkDir::new(dossier_parent).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let fichier_source_chemin_complet_string = entry.path().file_name().unwrap().to_str().unwrap();
        if fichier_source_chemin_complet_string.ends_with(".jpeg") || fichier_source_chemin_complet_string.ends_with(".jpg") {
            convertir_une_image_en_webp(&entry.path(), &dossier_cible, &compression);
        }
    }
    return dossier_cible;

}

fn convertir_une_image_en_webp(image_a_convertir: &Path, dossier_cible: &Path, compression: &f32) {
    // Chheck if target folder already exists, if not, web will refuse to convert.
    if !dossier_cible.exists() { fs::create_dir_all(dossier_cible); }
    let nom_fichier_cible = Path::new(image_a_convertir).file_stem().unwrap().to_str().unwrap();
    let chemin_absolu_vers_image_cible = Path::new(dossier_cible).join(nom_fichier_cible).with_extension("webp");

    let img = image::open(image_a_convertir).unwrap();
    let (w,h) = img.dimensions();
    // Optionally, resize the existing photo and convert back into DynamicImage
    let size_factor = 1.0;
    let img: DynamicImage = image::DynamicImage::ImageRgba8(
        imageops::resize(
            &img,
            (w as f64 * size_factor) as u32,
            (h as f64 * size_factor) as u32,
            imageops::FilterType::Triangle,)
    );

    // Create the WebP encoder for the above image
    let encoder: Encoder = Encoder::from_image(&img).unwrap();
    // Encode the image at a specified quality 0-100
    let webp: WebPMemory = encoder.encode(*compression);
    // Define and write the WebP-encoded file to a given path
    fs::write(chemin_absolu_vers_image_cible, &*webp).unwrap();
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

    let random_string: String = (0..taille)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    return random_string;
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
//
//
//
//
//
//


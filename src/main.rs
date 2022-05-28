extern crate core;
extern crate tar;
extern crate walkdir;

use std::env::args_os;
use std::{env, fs, io};
use std::collections::btree_map::Entry;
use std::fs::File;
use std::io::{copy, Read, stdin, stdout};
use std::path::Path;
use rand::Rng;
use unrar::Archive;
use walkdir::{DirEntry, WalkDir};
use image::*;
use webp::{Encoder, WebPMemory};


fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let accepted_input_file_extensions = ["cb7", "cba", "cbr", "cbt", "cbz"];

    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        // TODO : Put better explanations with examples.
        println!("Usage: program_name file_to_shrink output_file image_format image_compression");
        return;
    }

    // VERIFICATIONS
    if !Path::new(&args[1]).exists() { println!("Can not find this file : {:?}", &args[1]); return };
    if !Path::new(&args[1]).is_file() { println!("Not a file : {:?}", &args[1]); return };
    if !accepted_input_file_extensions.contains(&Path::new(&args[1]).extension().unwrap().to_str().unwrap().to_lowercase().as_str()) { println!("Input file extension not accepted {:?}", Path::new(&args[1]).extension().unwrap().to_str().unwrap().to_lowercase().as_str()); return;}

    // FIRST STEP : EXTRACTION
    let temporary_output_folder:String = ".".to_string() + &*random_string(10);
    match Path::new(&args[1]).extension().unwrap().to_str().unwrap().to_lowercase().as_str() {
        //"cb7" => extract_7zip_file(&args[1], &temporary_output_folder), // TODO Failed to create it
        //"cba" => extract_ace_file(&args[1], &temporary_output_folder), // TODO Failed to create it
        "cbr" => extract_rar_file(&args[1], &temporary_output_folder),
        //"cbt" => extract_tar_file(&args[1], &temporary_output_folder), // TODO Failed to create it
        "cbz" => extract_zip_file(&args[1], &temporary_output_folder),
        _ => println!("This archive file is not (yet) supported, sorry."),
    };

    let complete_file_list = get_complete_file_list(temporary_output_folder);
    match args[3].as_str() {
        //"avif" => convert_pictures_to_avif(&complete_file_list, &args[4]),
        //"mozjpg" => convert_pictures_to_mozjpp(&complete_file_list, &args[4]),
        "webp" => convert_pictures_to_webp(&complete_file_list, &args[4]),
        _ => println!("Can't convert pictures to this format : {}", &args[3])
    };
}

fn convert_pictures_to_webp(complete_file_list: &Vec<DirEntry>, compression: &String) {
    let extension_to_convert = ["jpeg", "jpg", "png", "webp"];
    for file in complete_file_list {
        if !file.path().exists() { continue; }
        let file_extension = file.path().extension().unwrap().to_str().unwrap();
        if !extension_to_convert.contains(&file_extension) { continue }

        let parent_directory = file.path().parent().unwrap();
        let filename = file.path().file_stem().unwrap();


        // READING THE ORIGINAL FILE
        let img = image::open(file.path()).unwrap();
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

        // WRITING THE DESTINATION FILE
        // Create the WebP encoder for the above image
        let encoder: Encoder = Encoder::from_image(&img).unwrap();
        // Encode the image at a specified quality 0-100
        let webp: WebPMemory = encoder.encode(compression.parse().unwrap());
        // Define and write the WebP-encoded file to a given path
        let output_file = Path::new(&parent_directory).join(&filename).with_extension("webp");
        fs::write(&output_file, &*webp).unwrap();

        // CLEANING
        fs::remove_file(file.path());
    }
}

fn get_complete_file_list(temporary_output_folder: String) -> Vec<DirEntry> {
    let mut file_list:Vec<DirEntry> = Vec::new();
    for e in WalkDir::new(temporary_output_folder).into_iter().filter_map(|e| e.ok()) {
        if e.metadata().unwrap().is_file() {
            file_list.push(e);
        }
    }
    return file_list;
}

fn extract_rar_file(file_to_extract: &String, temporary_output_folder: &String) {
    Archive::new(file_to_extract.parse().unwrap()).extract_to(temporary_output_folder.parse().unwrap()).unwrap().process().unwrap();
}

fn extract_zip_file(file_to_extract: &String, temporary_output_folder: &String) {
    let fname = Path::new(file_to_extract);
    let file = fs::File::open(&fname).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = Path::new(temporary_output_folder).join(file.sanitized_name());
        //println!("test: {:?}", outpath);

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            //println!("File {} extracted to \"{}\"", i, outpath.as_path().display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            //println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
}


fn random_string(taille: i32) -> String {
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


// use std::{env, fs};
// use std::ffi::OsStr;
// use std::fs::File;
// use std::io;
// use std::io::{Read, Seek, Write};
// use std::path::{Path, PathBuf};
// use rand::Rng;
//
//
// // todo extraire d'abord les 3 formats possibles ! avoir les mêmes retours ! garder la hiérarchie des dossiers
// // todo -> mettre les images à compresser dans un vecteur, si le format de destination est différent, supprimer ces fichiers à la fin.
//
// extern crate unrar;
// use unrar::Archive;
// use walkdir::{DirEntry, WalkDir};
// use image::*;
// use unrar::archive::Entry;
// use webp::{Encoder, WebPMemory};
// use zip::result::ZipError;
// use zip::write::FileOptions;
//
// extern crate zip;
// extern crate walkdir;
// extern crate core;
//
// const METHOD_STORED : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);
//
// #[cfg(feature = "deflate")]
// const METHOD_DEFLATED : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
// #[cfg(not(feature = "deflate"))]
// const METHOD_DEFLATED : Option<zip::CompressionMethod> = None;
//
// #[cfg(feature = "bzip2")]
// const METHOD_BZIP2 : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);
// #[cfg(not(feature = "bzip2"))]
// const METHOD_BZIP2 : Option<zip::CompressionMethod> = None;
//
//
//
// fn main() {
//     env::set_var("RUST_BACKTRACE", "full");
//     let formats_entrants_acceptes = ["cb7", "cbr", "cbz"];
//     let formats_sortants_acceptes = ["cb7", "cbz"];
//
//     // Bon, la ligne de commande ressemble à "nomDuProgramme NomDuFichier formatDuFichierSortant formatDesImages compression"
//     // TODO Vérifier le nombre d'arguments -> sinon afficher : Usage : "nomDuProgramme NomDuFichier formatDuFichier"
//     let args: Vec<String> = env::args().collect();
//     let fichier_source: &Path = Path::new(&args[1]);
//
//     let extension_fichier_sortant:String = args[2].clone();
//     let format_des_images:String = args[3].clone();
//     let compression_des_images:f32 = args[4].clone().parse::<f32>().unwrap();
//
//     // EXTRACTION DE L'ARCHIVE
//     let fichier_source_extension:String = extension_fichier(&fichier_source);
//     let fichier_source_nom:String = nom_fichier(&fichier_source);
//     let fichier_source_chemin_absolu:String = chemin_absolu_fichier(&fichier_source);
//     let fichier_source_dossier:String = dossier_fichier(&fichier_source);
//     if !fichier_source.exists() { println!("Le fichier n'existe pas"); return;}
//     if !formats_entrants_acceptes.contains(&&**&fichier_source_extension){ println!("Extension entrante non autorisée"); return; }
//     match fichier_source_extension.as_str() {
//          "cb7" => println!("cb7"),
//          "cbr" => compresser_dossier(&extension_fichier_sortant, &compresser_images(&format_des_images, &compression_des_images, extraire_archive_rar(&fichier_source_chemin_absolu, &fichier_source_dossier), &fichier_source_nom)),
//          "cbz" => extraire_archive_zip(&fichier_source_chemin_absolu, &fichier_source_dossier),
//          _ => println!("Extension entrante inconnue... encore ?"),
//     }
// }
//
//
// fn compresser_dossier(extension_fichier_sortant: &String, dossier_a_compresser: &PathBuf) {
//     for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2].iter() {
//         if method.is_none() { continue }
//         match doit(dossier_a_compresser.to_str().unwrap(), dossier_a_compresser.with_extension(extension_fichier_sortant).to_str().unwrap(), method.unwrap()) {
//             Ok(_) => println!("done: {} written to {}", dossier_a_compresser.to_str().unwrap(), dossier_a_compresser.with_extension(extension_fichier_sortant).to_str().unwrap()),
//             Err(e) => println!("Error: {:?}", e),
//         }
//     }
//     fs::remove_dir_all(dossier_a_compresser);
// }
//
// fn doit(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
//     if !Path::new(src_dir).is_dir() {
//         return Err(ZipError::FileNotFound);
//     }
//     let path = Path::new(dst_file);
//     let file = File::create(&path).unwrap();
//     let walkdir = WalkDir::new(src_dir.to_string());
//     let it = walkdir.into_iter();
//     zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;
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
//     let mut buffer = Vec::new();
//     for entry in it {
//         let path = entry.path();
//         let name = path.strip_prefix(Path::new(prefix)).unwrap();
//         // Write file or directory explicitly
//         // Some unzip tools unzip files with directory paths correctly, some do not!
//         if path.is_file() {
//             println!("adding file {:?} as {:?} ...", path, name);
//             zip.start_file_from_path(name, options)?;
//             let mut f = File::open(path)?;
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
// fn compresser_images(format_des_images: &String, compression_des_images: &f32, donnees_extraction: (String, Vec<Entry>), nom_archive: &String) -> PathBuf {
//     //println!("donnees extraction : {:?}", donnees_extraction);
//     let data = match format_des_images.as_str() {
//         "webp" => convertir_en_webp(&donnees_extraction.0, &compression_des_images, &nom_archive),
//         _ => panic!("Erreur d'ouverture du fichier : "),
//     };
//     return data;
// }
//
// fn convertir_en_webp(dossier_parent:&String, compression: &&f32, nom_archive: &String) -> PathBuf {
//     // Créons le dossier cible
//     let dossier_cible = Path::new(&dossier_parent).parent().unwrap().join(nom_archive);
//
//     for entry in WalkDir::new(dossier_parent).follow_links(true).into_iter().filter_map(|e| e.ok()) {
//         let fichier_source_chemin_complet_string = entry.path().file_name().unwrap().to_str().unwrap();
//         if fichier_source_chemin_complet_string.ends_with(".jpeg") || fichier_source_chemin_complet_string.ends_with(".jpg") {
//             convertir_une_image_en_webp(&entry.path(), &dossier_cible, &compression);
//         }
//     }
//     fs::remove_dir_all(dossier_parent);
//     return dossier_cible;
//
// }
//
// fn convertir_une_image_en_webp(image_a_convertir: &Path, dossier_cible: &Path, compression: &f32) {
//     // Chheck if target folder already exists, if not, web will refuse to convert.
//     if !dossier_cible.exists() { fs::create_dir_all(dossier_cible); }
//     let nom_fichier_cible = Path::new(image_a_convertir).file_stem().unwrap().to_str().unwrap();
//     let chemin_absolu_vers_image_cible = Path::new(dossier_cible).join(nom_fichier_cible).with_extension("webp");
//
//     let img = image::open(image_a_convertir).unwrap();
//     let (w,h) = img.dimensions();
//     // Optionally, resize the existing photo and convert back into DynamicImage
//     let size_factor = 1.0;
//     let img: DynamicImage = image::DynamicImage::ImageRgba8(
//         imageops::resize(
//             &img,
//             (w as f64 * size_factor) as u32,
//             (h as f64 * size_factor) as u32,
//             imageops::FilterType::Triangle,)
//     );
//
//     // Create the WebP encoder for the above image
//     let encoder: Encoder = Encoder::from_image(&img).unwrap();
//     // Encode the image at a specified quality 0-100
//     let webp: WebPMemory = encoder.encode(*compression);
//     // Define and write the WebP-encoded file to a given path
//     fs::write(chemin_absolu_vers_image_cible, &*webp).unwrap();
// }
//
//
// fn extraire_archive_rar(archive: &String, dossier_sortie: &String) -> (String, Vec<Entry>) {
//     let chaine_aleatoire = chaine_aleatoire(10);
//     return (
//         String::from(Path::new(dossier_sortie).join(&chaine_aleatoire).to_str().unwrap()),
//         Archive::new(archive.to_string()).extract_to(String::from(Path::new(dossier_sortie).join(&chaine_aleatoire).to_str().unwrap()).parse().unwrap()).unwrap().process().unwrap()
//     );
// }
//
// fn extraire_archive_zip(archive: &String, dossier_sortie: &String) {
//     let file = fs::File::open(&archive).unwrap();
//
//     let mut archive = zip::ZipArchive::new(file).unwrap();
//
//     for i in 0..archive.len() {
//         println!("{:?}", archive.i);
//         let mut file = archive.by_index(i).unwrap();
//         let outpath = Path::new(dossier_sortie);
//
//         {
//             let comment = file.comment();
//             if !comment.is_empty() {
//                 println!("File {} comment: {}", i, comment);
//             }
//         }
//
//         if (&*file.name()).ends_with('/') {
//             println!("File {} extracted to \"{}\"", i, outpath.display());
//             fs::create_dir_all(&outpath).unwrap();
//         } else {
//             println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.display(), file.size());
//             if let Some(p) = outpath.parent() {
//                 if !p.exists() {
//                     fs::create_dir_all(&p).unwrap();
//                 }
//             }
//             let mut outfile = fs::File::create(&outpath).unwrap();
//             io::copy(&mut file, &mut outfile).unwrap();
//         }
//
//         // Get and Set permissions
//         #[cfg(unix)]
//         {
//             use std::os::unix::fs::PermissionsExt;
//
//             if let Some(mode) = file.unix_mode() {
//                 fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
//             }
//         }
//     }
//
//     let chaine_aleatoire = chaine_aleatoire(10);
//     //return (String::from(""),Vec(Entry)
//         //String::from(Path::new(dossier_sortie).join(&chaine_aleatoire).to_str().unwrap()),
//         //Archive::new(archive.to_string()).extract_to(String::from(Path::new(dossier_sortie).join(&chaine_aleatoire).to_str().unwrap()).parse().unwrap()).unwrap().process().unwrap()
//     //);
// }
//
//
// fn chaine_aleatoire(taille: i32) -> String {
//     const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
//     let mut rng = rand::thread_rng();
//
//     let random_string: String = (0..taille)
//         .map(|_| {
//             let idx = rng.gen_range(0..CHARSET.len());
//             CHARSET[idx] as char
//         })
//         .collect();
//     return random_string;
// }
//
// fn chemin_absolu_fichier(fichier: &Path) -> String {
//     return String::from(fichier.clone().canonicalize().unwrap().to_str().unwrap());
// }
//
// fn extension_fichier(fichier: &Path) -> String {
//     return String::from(fichier.clone().extension().and_then(OsStr::to_str).unwrap_or("extension entrante non reconnue"));
// }
//
// fn nom_fichier(fichier: &Path) -> String {
//     return String::from(fichier.clone().file_stem().and_then(OsStr::to_str).unwrap_or("nom fichier entrant non reconnue"));
// }
//
// fn dossier_fichier(fichier: &Path) -> String {
//     return String::from(fichier.parent().unwrap().to_str().unwrap_or("dossier parent introuvable"));
// }
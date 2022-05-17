use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;


extern crate unrar;
use unrar::Archive;
use walkdir::{DirEntry, WalkDir};
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
    let formats_entrants_acceptes = ["cbz", "cb7"];
    let formats_sortants_acceptes = ["cbz", "cb7"];

    // Bon, la ligne de commande ressemble à "nomDuProgramme NomDuFichier formatDuFichier"
    // TODO Vérifier le nombre d'arguments -> sinon afficher : Usage : "nomDuProgramme NomDuFichier formatDuFichier"
    let args: Vec<String> = env::args().collect();
    let nom_fichier_source = &args[1];
    let extension_fichier_sortant = args[2].clone();

    // VÉRIFICATIONS : // TODO Mettre un message d'erreur différent à chaque vérification.
    // Le fichier existe-t-il ?
    let fichier_existant : bool = fichier_existe(nom_fichier_source);
    println!("Le fichier existe-t-il : {}", fichier_existant);
    // L'extension du fichier est-elle acceptée ?
    let extension_fichier_entrant : String = extension_fichier(nom_fichier_source);
    println!("L'extension trouvée : {:?}", extension_fichier_entrant);
    let extension_entrante_valide = extension_acceptee(formats_entrants_acceptes, &extension_fichier_entrant);
    println!("L'extension entrante est valide : {}", extension_entrante_valide);
    // L'extension du fichier de sortie est-elle valide ?
    let extension_sortante_valide = extension_acceptee(formats_sortants_acceptes, &extension_fichier_sortant);
    println!("L'extension sortante est valide : {}", extension_sortante_valide);
    if extension_sortante_valide && extension_entrante_valide && fichier_existant {
        println!("on lance !");
        // C'est parti : -> on extraie les fichiers -> on compresse les fichiers -> on supprime les fichiers-> c'est tout

        // 1° EXTRACTION
        let fichier_test = "/home/sacha/Projets/rs_comic_shrinker/comics/test_cbr.cbr";
        extraire_archive(extension_fichier_entrant, &fichier_test);
        // 2° COMPRESSION
        let src_dir = "/home/sacha/Projets/rs_comic_shrinker/comics/extracted/berserk_test";
        let dst_file = "/home/sacha/Projets/rs_comic_shrinker/comics/test.cbz";
        for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2].iter() {
            if method.is_none() { continue }
            match doit(src_dir, dst_file, method.unwrap()) {
                Ok(_) => println!("done: {} written to {}", src_dir, dst_file),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
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


fn extraire_archive(extension_fichier_entrant: String, fichier_test:&str){
    println!("L'extension à extraire : {:?}", &extension_fichier_entrant);
    // Get the archive information and extract everything
    let test = String::from(fichier_test);
    Archive::new(test).extract_to("/home/sacha/Projets/rs_comic_shrinker/comics/extracted/".parse().unwrap()).unwrap().process().unwrap();
    println!("Done.");
}

fn extension_acceptee(formats_acceptes: [&str; 2], extension_fichier: &String) ->bool{
    return formats_acceptes.contains(&&**extension_fichier);
}

fn extension_fichier(nom_fichier: &str) -> String {
    return String::from(Path::new(nom_fichier).extension().and_then(OsStr::to_str).as_deref().unwrap_or("extension inconnue"));
}

fn fichier_existe(nom_fichier_source: &String)->bool {
    return Path::new(nom_fichier_source).exists()
}
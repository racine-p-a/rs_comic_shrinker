extern crate core;
extern crate tar;
extern crate walkdir;

use std::{env, fs, io};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use rand::Rng;
use unrar::Archive;
use walkdir::{DirEntry, WalkDir};
use image::*;
use webp::{Encoder, WebPMemory};
use zip::result::ZipError;
use zip::write::FileOptions;

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
    let accepted_input_file_extensions = ["cb7", "cba", "cbr", "cbt", "cbz"];
    let accepted_destination_file_extensions = ["cbz"];

    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("rs_comic_shrinker path/to/big/comic.cbr /path/to/desired/shrinked/comicc.cbz webp 1");
        println!("Actually, you can only shrink CBZ and CBR files. More to come.");
        println!("Actually, you will get only CBZ shrinked files. More to come.");
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

    // SECOND STEP : IMAGE COMPRESSION
    let complete_file_list = get_complete_file_list(&temporary_output_folder);
    match args[3].as_str() {
        //"avif" => convert_pictures_to_avif(&complete_file_list, &args[4]),
        //"mozjpg" => convert_pictures_to_mozjpp(&complete_file_list, &args[4]),
        "webp" => convert_pictures_to_webp(&complete_file_list, &args[4]),
        _ => println!("Can't convert pictures to this format : {}", &args[3])
    };

    // THIRD STEP : ARCHIVE CREATION
    if !accepted_destination_file_extensions.contains(&Path::new(&args[2]).extension().unwrap().to_str().unwrap()) { println!("Output file extension not accepted {:?}", &Path::new(&args[2]).extension().unwrap().to_str().unwrap()); return;}
    let output_filename = Path::new(&args[2]).file_stem().unwrap().to_str().unwrap();
    match Path::new(&args[2]).extension().unwrap().to_str().unwrap() {
        "cbz" => zip_folder(&temporary_output_folder, Path::new(&output_filename).with_extension("cbz").to_str().unwrap()),
        _ => println!("Can't compress to this format (yet) : {}", Path::new(&args[2]).extension().unwrap().to_str().unwrap())
    };
}

fn zip_folder(temporary_output_folder: &String, output_filename: &str) {
    for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2].iter() {
        if method.is_none() { continue }
        match doit(&temporary_output_folder.as_str(), &output_filename, method.unwrap()) {
            Ok(_) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }
    match fs::remove_dir_all(&temporary_output_folder) {
        Ok(_) => (),
        Err(e) => println!("Error: {:?}", e),
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
            //println!("adding file {:?} as {:?} ...", path, name);
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            //println!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
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
        let img = open(file.path()).unwrap();
        let (w,h) = img.dimensions();
        // Optionally, resize the existing photo and convert back into DynamicImage
        let size_factor = 1.0;
        let img: DynamicImage = DynamicImage::ImageRgba8(
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
        match fs::remove_file(file.path()) {
            Ok(_) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}

fn get_complete_file_list(temporary_output_folder: &String) -> Vec<DirEntry> {
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
    let file = File::open(&fname).unwrap();
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
            let mut outfile = File::create(&outpath).unwrap();
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

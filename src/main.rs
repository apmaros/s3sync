mod store;
mod file;
mod model;
mod sync_error;
mod cli;

extern crate google_cloud;

use crate::file::list_files;
use crate::model::photo_album::ImageAlbum;
use crate::model::photo::Photo;
use crate::store::{get_client, list_bucket_names};

use std::path::{PathBuf};
use std::fs::File;
use std::io::{Read, Write, stdout, Stdout};
use std::process::exit;
use crossterm::{QueueableCommand, cursor};
use magick_rust::{magick_wand_genesis, MagickWand};
use std::sync::Once;
use crate::sync_error::SyncError;
use std::str::FromStr;
use crate::cli::{build_cli, CliCommand, UploadCmd};
use crate::cli::CliCommand::UPLOAD;
use google_cloud::storage::Bucket;

type GenError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() {
    let matches = build_cli();

    let cmd = CliCommand::from_str(
        matches.subcommand_name().unwrap_or_else(|| {
            eprintln!("No parameter was provided");
            exit(1);
    }));

    let result = match cmd {
        Ok(CliCommand::UPLOAD) => {
            let c = UploadCmd::build(
                matches.subcommand_matches(UPLOAD.to_str()).unwrap()
            );
            let mut album_name = "apmaros_".to_owned();
            album_name.push_str(&c.album_name);
            upload_images(&c.folder_name, &album_name, c.downscale).await
        }
        Ok(CliCommand::LIST) => print_bucket_names().await,
        Err(invalid_cmd) => Err(
            GenError::from(SyncError(format!("Command {} is not valid", invalid_cmd).to_string().into())
        ))
    };

    println!();
    match result {
        Ok(_) => println!("Success 🎉"),
        Err(err) => {
            eprintln!("❌  Failed due to {}", err);
            exit(1);
        }
    }
}

async fn print_bucket_names() -> Result<(), GenError>{
    let bucket_names = list_bucket_names().await?;

    println!("🧺 found {} buckets:", bucket_names.len());

    let limit = 100;
    let buckets_to_print = if bucket_names.len() > limit {
        println!("Too many bucket to print, first {} will be printed", limit);
        &bucket_names[0..limit]
    } else { &bucket_names };

    buckets_to_print.iter().for_each(|b| println!("\t{}", b));

    Ok(())
}

async fn upload_images(folder_name: &str, album_name: &str, downscale: bool) -> Result<(), GenError>{
    let photos = read_photos(list_files(folder_name))?;
    println!("loaded {} photos", photos.len());

    if photos.len() == 0 {
        println!("No photos to add");
        exit(0);
    }

    if downscale { println!("⚠️  Images will stored in lower size and resolution");
    } else { println!("Images will be stored in original resolution")}

    let album_name_s = album_name.parse().unwrap();
    let album = ImageAlbum { name: album_name_s, photos, downscale};
    let blocking_task = tokio::task::spawn_blocking(move || {
        upload_images_to_cloud(album)
    }).await;

    match blocking_task {
        Ok(task) => task.await,
        Err(err) => Err(GenError::from(err))
    }
}

async fn upload_images_to_cloud(album: ImageAlbum) -> Result<(), GenError>{
    let bucket = create_bucket(&album.name).await?;
    upload_photos(bucket, album).await
}

async fn create_bucket(name: &str) -> Result<Bucket, GenError>{
    let mut client = get_client().await?;

    println!("Creating bucket 🧺 {}", &name);
    match client.create_bucket(name).await {
        Ok(bucket) => Ok(bucket),
        Err(err) => {
            eprintln!("Failed to create bucket {} due to error", &name);
            return Err(GenError::from(err))
        }
    }
}

async fn upload_photos(mut bucket: Bucket, album: ImageAlbum) -> Result<(), GenError> {
    let stdout = &stdout();

    for (i, photo) in album.photos.iter().enumerate(){
        let mut file = File::open(&photo.path)?;
        let mut buffer = vec![0; photo.metadata.len() as usize];
        file.read(&mut buffer)?;

        let image_data = if album.downscale {
            resize(&buffer)? } else { buffer };

        match bucket.create_object(&photo.name, image_data, "image/jpeg").await {
            Ok(_) => {
                rewrite_message(&stdout, format!("uploaded {} / {} files", i+1, album.photos.len()))?
            },
            Err(err) => {
                println!("failed to create object due to {:?}", err);
            }
        };
    }
    println!(); // start new line
    Ok(())
}
static START: Once = Once::new();

fn resize(data: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let wand = MagickWand::new();
    wand.read_image_blob(data)?;

    let new_width = wand.get_image_width() / 5;
    let new_height = wand.get_image_height() / 5;
    wand.adaptive_resize_image(new_width as usize, new_height as usize)?;

    let (res_x, res_y) = wand.get_image_resolution()?;
    let new_res_x = res_x * 0.55;
    let new_res_y = res_y * 0.55;

    wand.set_resolution(new_res_x, new_res_y)?;

    wand.write_image_blob("jpeg")
}

fn rewrite_message(mut stdout: &Stdout, msg: String) -> Result<(), GenError> {
    stdout.queue(cursor::SavePosition)?;
    stdout.write(msg.as_bytes())?;
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn read_photos(paths: Vec<PathBuf>) -> Result<Vec<Photo>, GenError> {
    let mut photos = Vec::new();

    for path in paths {
        let file = File::open(&path)?;
        let meta = file.metadata()?;
        let extension = match path.extension() {
            Some(extension) => extension.to_str().unwrap().to_lowercase(),
            None => "".to_string()
        };

        let filename = path.file_name().unwrap().to_str().unwrap();
        if meta.is_file() && extension == "jpg" {
            let photo = Photo {
                name: String::from(filename),
                path: Box::from(path),
                metadata: meta
            };
            photos.push(photo);
        } else {
            println!("{:?} is not an image", &path)
        }
    }

    Ok(photos)
}

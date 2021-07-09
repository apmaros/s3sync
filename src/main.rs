mod file;
mod model;
mod sync_error;
mod cli;
mod s3_store;
mod utils;

use crate::file::list_files;
use crate::model::photo_album::ImageAlbum;
use crate::model::photo::Photo;
use crate::sync_error::SyncError;
use crate::cli::{build_cli, CliCommand, UploadCmd, DownloadCmd};
use crate::cli::CliCommand::{UPLOAD, DOWNLOAD};
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::{Read, Write, stdout};
use std::process::exit;
use std::str::FromStr;
use crate::s3_store::{StoreClient};
use crate::utils::rewrite_message;

type GenError = Box<dyn std::error::Error>;
static BUCKET_NAME: &str = "apmaros-store";
static IMAGE_EXTENSION: &str = "jpg";

#[tokio::main]
async fn main() {
    let matches = build_cli();
    let store = StoreClient::new().unwrap_or_else(|err| {
        eprintln!("Failed to create store client due to error = {}", err);
        exit(1);
    });

    let cmd = CliCommand::from_str(
        matches.subcommand_name().unwrap_or_else(|| {
            eprintln!("No parameter was provided, run `cloud help` to learn more");
            exit(1);
    }));

    let result = match cmd {
        Ok(CliCommand::UPLOAD) => {
            let c = UploadCmd::build(
                matches.subcommand_matches(UPLOAD.to_str()).unwrap()
            );

            upload(store.clone(), &c.folder_name, &c.album_name).await
        }
        Ok(CliCommand::LIST) => list_albums(store).await,
        Ok(CliCommand::DOWNLOAD) => {
            let c = DownloadCmd::build(
                matches.subcommand_matches(DOWNLOAD.to_str()).unwrap()
            );
            download(store, c.album_name).await
        },
        Err(invalid_cmd) => Err(
            GenError::from(SyncError(format!("Command {} is not valid", invalid_cmd).to_string().into())
        ))
    };

    println!();
    match result {
        Ok(_) => exit(0),
        Err(err) => {
            eprintln!("❌  Failed due to error='{}'", err);
            exit(1);
        }
    }
}

async fn download(store: StoreClient, album_name: String) -> Result<(), GenError> {
    let keys = store.list_keys(
        BUCKET_NAME,
        Some(album_name.as_str())
    ).await?;

    for (i, key) in keys.iter().enumerate() {
        let body = store.get_object(BUCKET_NAME, key.as_str()).await?;

        if body.len() > 0 {
            let filename: String = key.as_str().to_string().split("/").collect();
            let path = Path::new(filename.as_str());
            File::create(&path).and_then(|mut f| f.write_all(&*body))?;
        } else { println!("Key {} does not have any data and wont be downloaded", key) }

        rewrite_message(stdout(), format!("Downloaded {} / {} files",   i+1, keys.len()))?;
    }

    Ok(())
}

async fn list_albums(store: StoreClient) -> Result<(), GenError>{
    let album_names = store.list_folders(
        BUCKET_NAME.to_string(),
        None
    ).await.unwrap();

    println!("📚 found {} albums:", album_names.len());

    let limit = 100;
    let buckets_to_print = if album_names.len() > limit {
        println!("Too many albums to print, first {} will be printed", limit);
        &album_names[0..limit]
    } else { &album_names };

    buckets_to_print.iter().for_each(|b| println!("\t{}", b));

    Ok(())
}

async fn upload(store: StoreClient, folder_name: &str, album_name: &str) -> Result<(), GenError>{
    let photos = read_photos(list_files(folder_name))?;
    println!("loaded {} photos", photos.len());

    if photos.len() == 0 {
        println!("No photos to add");
        exit(0);
    }

    let name = album_name.parse().unwrap();
    let album = ImageAlbum { name, photos };
    let blocking_task = tokio::task::spawn_blocking(move || {
        upload_album(store, album)
    }).await;

    match blocking_task {
        Ok(task) => task.await,
        Err(err) => Err(GenError::from(err))
    }
}

async fn upload_album(client: StoreClient, album: ImageAlbum) -> Result<(), GenError> {
    for (i, photo) in album.photos.iter().enumerate(){
        let mut file = File::open(&photo.path)?;
        let mut buffer = vec![0; photo.metadata.len() as usize];
        file.read(&mut buffer)?;

        let key = format!("{}/{}", album.name, photo.name);
        match client.put(key, &buffer, BUCKET_NAME.to_owned()).await {
            Ok(_) => {
                rewrite_message(stdout(), format!("uploaded {} / {} files", i+1, album.photos.len()))?
            },
            Err(err) => {
                println!("failed to create object due to {:?}", err);
            }
        };
    }
    println!();
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
        if meta.is_file() && extension == IMAGE_EXTENSION {
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

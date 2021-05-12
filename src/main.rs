mod store;
mod file;
mod model;

extern crate google_cloud;
use std::env;
use std::path::{PathBuf};
use crate::file::list_files;
use crate::model::photo_album::PhotoAlbum;
use crate::model::photo::Photo;
use crate::store::get_client;
use std::fs::File;
use std::io::{Read, Write, stdout, Stdout};
use std::process::exit;
use std::error::Error;
use std::fmt;
use crossterm::{QueueableCommand, cursor};
use google_cloud::storage::Bucket;

type GenError = Box<dyn std::error::Error>;

#[derive(Debug)]
struct SyncError(String);
impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for SyncError {}


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let folder_name = &args[1];
    let mut album_name = "apmaros_".to_owned();
    println!("Listing files for {}", folder_name);
    album_name.push_str(&args[2]);

    let photos = read_photos(list_files(folder_name));

    println!("loaded {} photos", photos.len());

    if photos.len() == 0 {
        println!("No photos to add");
        exit(0);
    }

    let blocking_task = tokio::task::spawn_blocking(|| {
        upload_to_cloud(PhotoAlbum { name: String::from(album_name), photos })
    }).await;

    let result = match blocking_task {
        Ok(task) => task.await,
        Err(err) => Err(GenError::from(err))
    };

    match result {
        Ok(_) => println!("Finished uploading photos"),
        Err(err) => exit_with_error(err)
    }
}

async fn upload_to_cloud(album: PhotoAlbum) -> Result<(), GenError>{
    let mut client = get_client().await?;
    match client.bucket(&album.name).await {
        Ok(_) => exit_with_error(
            Box::new(SyncError(format!("Folder {} already exists, choose different name", &album.name).to_string().into())
        )).into(),
        Err(_) => {
            println!("Creating bucket 🧺 {}", &album.name);
            match client.create_bucket(&album.name).await {
                Ok(bucket) => upload_photos(bucket, album.photos).await?,
                Err(err) => {
                    eprintln!("Failed to create bucket {} due to {:?}", &album.name, err);
                    return Err(GenError::from(err))
                }
            }
        }
    }
    Ok(())
}

async fn upload_photos(mut bucket: Bucket, photos: Vec<Photo>) -> Result<(), GenError> {
    let stdout = &stdout();

    for (i, photo) in photos.iter().enumerate(){

        let mut file = File::open(&photo.path)?;
        let mut buffer = vec![0; photo.metadata.len() as usize];
        file.read(&mut buffer).expect("Failed to read file");

        match bucket.create_object(&photo.name, buffer, "image/jpeg").await {
            Ok(_) => {
                rewrite_message(&stdout, format!("uploaded {} / {} files", i, photos.len())).unwrap()
            },
            Err(err) => {
                println!("failed to create object due to {:?}", err);
            }
        };
    }

    Ok(())
}

fn rewrite_message(mut stdout: &Stdout, msg: String) -> Result<(), GenError> {
    stdout.queue(cursor::SavePosition)?;
    stdout.write(msg.as_bytes())?;
    stdout.queue(cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn read_photos(paths: Vec<PathBuf>) -> Vec<Photo> {
    let mut photos = Vec::new();

    for path in paths {
        let file = File::open(&path).expect("Failed to open file");
        let meta = file.metadata().expect("Can not read metadata");
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

    photos
}

fn exit_with_error(err: GenError) {
    eprintln!("Exiting with error {}", err);
    exit(1);
}

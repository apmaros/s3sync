mod store;
mod file;
mod model;

extern crate google_cloud;
use futures::executor::block_on;
use std::env;
use std::path::{PathBuf};
use crate::file::list_files;
use crate::model::photo_album::PhotoAlbum;
use crate::model::photo::Photo;
use crate::store::get_client;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let folder_name = &args[1];
    let mut album_name = "apmaros_".to_owned();
    println!("Listing files for {}", folder_name);
    album_name.push_str(&args[2]);

    let photos = read_photos(list_files(folder_name));

    println!("loaded {} photos", photos.len());

    if photos.len() == 0 { println!("No photos to add") }
    else { block_on(upload_to_cloud( PhotoAlbum {name: String::from(album_name), photos})) };
}

async fn upload_to_cloud(album: PhotoAlbum) {
    let mut client = get_client().await.expect("Failed to get client");
    match client.bucket(&album.name).await {
        Ok(_) => println!("Folder already exists, choose new name"),
        Err(_) => {
            println!("Creating bucket 🧺");
            match client.create_bucket(&album.name).await {
                Ok(mut bucket) => {
                    for photo in album.photos {
                        bucket.create_object(&photo.name, photo.content, "image/jpeg").await;
                        println!("uploaded {}", &photo.name)
                    }
                },
                Err(err) => eprintln!("Failed to create bucket {} due to {:?}", &album.name, err)
            }
        }
    }
}

fn read_photos(paths: Vec<PathBuf>) -> Vec<Photo> {
    let mut photos = Vec::new();

    for path in paths {
        println!("reading: {:?}", &path);
        let mut file = File::open(&path).expect("Failed to open file");
        let meta = file.metadata().expect("Can not read metadata");
        let mut buffer = vec![0; meta.len() as usize];
        let extension = match path.extension() {
            Some(extension) => extension.to_str().unwrap().to_lowercase(),
            None => "".to_string()
        };

        let filename = path.file_name().unwrap().to_str().unwrap();
        if meta.is_file() && extension == "jpg" {
            file.read(&mut buffer).expect("Failed to read file");

            let photo = Photo {
                name: String::from(filename),
                path: Box::from(path),
                content: buffer,
                metadata: meta
            };
            photos.push(photo);
        } else {
            println!("{:?} is not an image", &path)
        }
    }

    photos
}

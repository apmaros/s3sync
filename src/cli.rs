// TODO make a folder

use std::str::FromStr;
use clap::{ArgMatches, App, Arg};

static VERSION: &str = "0.0.1";
static AUTHOR: &str = "apmaros";
static DESCRIPTION: &str = "A set of useful tools for fun and profit";
const UPLOAD: &str = "upload";
const UPLOAD_SHORT: &str = "u";
const LIST: &str = "list";
const DOWNLOAD: &str = "download";
const DOWNLOAD_SHORT: &str = "d";
pub(crate) const FOLDER: &str = "folder";
const FOLDER_SHORT: &str = "folder";
pub(crate) const ALBUM: &str = "album";
const ALBUM_SHORT: &str = "a";
pub(crate) const DOWNSCALE: &str = "downscale";
const DOWNSCALE_SHORT: &str = "d";

pub(crate) fn build_cli<'a>() -> ArgMatches<'a> {
    App::new("Upload files to cloud")
        .version(VERSION)
        .author(AUTHOR)
        .about(DESCRIPTION)
        .subcommand(App::new(UPLOAD)
            .version_short(UPLOAD_SHORT)
            .help("Uploads images to cloud")
            .arg(Arg::with_name(FOLDER)
                .short(FOLDER_SHORT)
                .long(FOLDER)
                .takes_value(true)
                .help("Folder containing images to be uploaded")
                .required(true))
            .arg(Arg::with_name(ALBUM)
                .short(ALBUM_SHORT)
                .long(ALBUM)
                .takes_value(true)
                .help("Folder name in cloud to upload images into (will be prepended with apmaros)")
                .required(true))
            .arg(Arg::with_name(DOWNSCALE)
                .short(DOWNSCALE_SHORT)
                .long(DOWNSCALE)
                .takes_value(false)
                .help("Downscales images")))
        .subcommand(App::new(DOWNLOAD)
            .version_short(DOWNLOAD_SHORT)
            .help("Downloads files from cloud")
            .arg(Arg::with_name(ALBUM)
                .short(ALBUM_SHORT)
                .long(ALBUM)
                .takes_value(true)
                .help("Album containing files to be downloaded")
                .required(true)))
        .subcommand(App::new(LIST))
        .get_matches()
}

pub(crate) enum CliCommand {
    UPLOAD,
    DOWNLOAD,
    LIST
}

impl FromStr for CliCommand {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            UPLOAD => Ok(Self::UPLOAD),
            LIST => Ok(Self::LIST),
            DOWNLOAD => Ok(Self::DOWNLOAD),
            _ => Err("Command {} was not recognised")
        }
    }
}

// TODO use display instead
impl CliCommand {
    pub(crate) fn to_str(&self) -> &str {
        match self {
            CliCommand::UPLOAD => UPLOAD,
            CliCommand::LIST => LIST,
            CliCommand::DOWNLOAD => DOWNLOAD
        }
    }
}

pub(crate) struct UploadCmd {
    pub(crate) folder_name: String,
    pub(crate) album_name: String
}

impl UploadCmd {
    pub(crate) fn build(matches: &ArgMatches) -> Self {
        // safe to unwrap, these args are required
        let folder_name = matches.value_of(FOLDER).unwrap().to_owned();
        let album_name = matches.value_of(ALBUM).unwrap().to_owned();

        UploadCmd { folder_name, album_name }
    }
}

pub(crate) struct DownloadCmd {
    pub album_name: String
}

impl DownloadCmd {
    pub(crate) fn build(matches: &ArgMatches) -> Self {
        let album_name = matches.value_of(ALBUM).unwrap().to_owned();

        DownloadCmd { album_name }
    }
}
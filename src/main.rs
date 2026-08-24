use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use lofty::error::FileParseError;
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::{Accessor, ItemKey};
use walkdir::WalkDir;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: String,

    #[arg(short, long, value_name = "DESTINATION")]
    dest: String,

    #[arg(long)]
    doit: bool,

    #[arg(long)]
    copy: bool,
}

// ext/Artist/Album/tracknumber-trackname.ext
fn read_tags(path: &Path) -> Result<HashMap<&str, Option<String>>, FileParseError> {
    let tagged_file = read_from_path(path)?;
    // || is an anonymous function that takes in this cas no arguments to execute the function,
    // which is everything that follows.
    let tags = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let mut map = HashMap::new();
    let artist = tags
        .and_then(|t| t.get_string(ItemKey::AlbumArtist))
        .map(|s| s.to_string());
    let album = tags.and_then(|t| t.album().map(|s| s.to_string()));
    let tracknumber = tags.and_then(|t| t.track().map(|s| s.to_string()));
    let title = tags.and_then(|t| t.title().map(|s| s.to_string()));
    let ext = path.extension().map(|t| t.to_string_lossy().to_string());

    map.insert("artist", artist);
    map.insert("album", album);
    map.insert("tracknumber", tracknumber);
    map.insert("title", title);
    map.insert("ext", ext);

    Ok(map)
}

fn build_path(taglist: &HashMap<&str, Option<String>>, target: PathBuf) -> PathBuf {
    let ext = taglist["ext"]
        .clone()
        .unwrap_or("[Unknown Extension]".to_string());
    let artist = taglist["artist"]
        .clone()
        .unwrap_or("[Unknown Artist]".to_string());
    let album = taglist["album"]
        .clone()
        .unwrap_or("[Unknown Album]".to_string());

    let ext = sanitize_filename(&ext);
    let artist = sanitize_filename(&artist);
    let album = sanitize_filename(&album);

    target.join(ext).join(artist).join(album)
}

fn to_asolute(path: &PathBuf) -> std::io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '"' | '<' | '>' | '|' => '_',
            '?' => 'q',
            _ => c,
        })
        .collect()
}

fn main() {
    let args = Args::parse();
    let source = args.source;
    let dest = args.dest;

    let doit = args.doit;
    let copy = args.copy;

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let e = read_tags(entry.path());
        // println!("{}", entry.path().display());
        match e {
            Ok(list) => {
                let target_path = build_path(&list, PathBuf::from(&dest));
                let tracknumber = list["tracknumber"].clone().unwrap_or("00".to_string());
                let title = list["title"]
                    .clone()
                    .expect("Track with no title ??? please check");
                let ext = list["ext"].clone().expect("Track with no extension ???");

                let tracknumber = sanitize_filename(&tracknumber);
                let title = sanitize_filename(&title);

                let target_name = target_path.join(format!("{}-{}.{}", tracknumber, title, ext));
                let absolute_path = to_asolute(&target_name).expect("Failed to make path absolute");
                // println!("{}", absolute_path.display());
                if doit {
                    if copy {
                        fs::create_dir_all(target_path).expect("Failed to create target path");
                        fs::copy(entry.into_path(), absolute_path).expect("failed to copy file");
                    } else {
                        fs::create_dir_all(target_path).expect("Failed to create target path");

                        fs::rename(entry.into_path(), absolute_path)
                            .expect("Failed to rename file");
                    }
                } else {
                    println!("This was a Dry Run add --doit to run it for real. Make a backup!!!")
                }
            }
            Err(er) => println!("{}, file: {:?}", er, entry.path().to_str()),
        }
    }
}

use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use metaflac::Tag;
use walkdir::WalkDir;
mod error;
mod metaflac_streaming;

fn flacs<P: AsRef<Path>>(path: P) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|res| match res {
            Ok(e)
                if e.file_type().is_file()
                    && e.file_name()
                        .to_str()
                        .filter(|s| s.ends_with("flac"))
                        .is_some() =>
            {
                Some(e.into_path())
            }
            _ => None,
        })
}

fn metaflac_count<I: Iterator<Item = PathBuf>>(file_iter: I) -> u32 {
    let mut count = 0;
    for path in file_iter {
        if let Ok(tag) = Tag::read_from_path(path) {
            if let Some(vc) = tag.vorbis_comments() {
                for (_, vs) in &vc.comments {
                    for v in vs {
                        if v == "Miles Davis" {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

fn metaflac_streaming_count<I: Iterator<Item = PathBuf>>(file_iter: I) -> u32 {
    let mut buf = Vec::new();
    let mut count = 0;
    for path in file_iter {
        if let Ok(mut vc) = metaflac_streaming::read_from(path, &mut buf) {
            while vc.next(&buf) {
                if let Ok(Some((_, _, v))) = vc.cur(&buf) {
                    if v == "Miles Davis" {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];

    let now = Instant::now();
    let count = metaflac_count(flacs(path));
    println!(
        "metaflac_count returns {} results in {}s.",
        count,
        now.elapsed().as_secs_f64()
    );

    let now = Instant::now();
    let count = metaflac_streaming_count(flacs(path));
    println!(
        "metaflac_streaming_count returns {} results in {}s.",
        count,
        now.elapsed().as_secs_f64()
    );
}

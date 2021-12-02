use crate::error::Error;
use mpdrs::lsinfo::LsInfoResponse;

pub fn complete(client: &mut mpdrs::Client, search_path: &str) -> Result<(), Error> {
    let prefix_path = match search_path.rfind('/') {
        Some(i) => &search_path[..i],
        None => "",
    };
    for item in client.lsinfo(prefix_path)? {
        match item {
            LsInfoResponse::Song(s) if s.file.starts_with(search_path) => println!("{}", s.file),
            LsInfoResponse::Playlist { path, .. } | LsInfoResponse::Directory { path, .. }
                if path.starts_with(search_path) =>
            {
                println!("{}", path);
            }
            _ => (),
        }
    }
    Ok(())
}

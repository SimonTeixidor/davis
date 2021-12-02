use std::env;
use std::ffi;
use std::fs;
use std::iter::once;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn find_subcommand(name: &ffi::OsStr) -> Option<PathBuf> {
    let path_str = env::var("PATH").expect("$PATH was not set!");
    let home = env::var("HOME").expect("$HOME was not set!");
    let home_subcommands: PathBuf = [&*home, ".config", "davis", "bin"].iter().collect();
    let etc_subcommands: PathBuf = ["/", "etc", "davis", "bin"].iter().collect();
    let custom_dirs = once(home_subcommands).chain(once(etc_subcommands));
    let paths = env::split_paths(&*path_str)
        .chain(custom_dirs)
        .collect::<Vec<_>>();

    let mut binary_name = ffi::OsString::from("davis-");
    binary_name.push(name);

    log::trace!(
        "Searching for subcommand with name {:?} in paths {:?}",
        binary_name,
        paths
    );

    paths
        .into_iter()
        .flat_map(|p| fs::read_dir(p).into_iter().flatten())
        .flat_map(IntoIterator::into_iter)
        .find(|d| d.file_name() == binary_name && is_executable(d))
        .map(|d| d.path())
}

// copied from https://github.com/frewsxcv/rust-quale
static EXECUTABLE_FLAGS: u32 = (libc::S_IEXEC | libc::S_IXGRP | libc::S_IXOTH) as u32;
fn is_executable(file: &fs::DirEntry) -> bool {
    // Don't use `file.metadata()` directly since it doesn't follow symlinks.
    let file_metadata = match file.path().metadata() {
        Ok(metadata) => metadata,
        Err(..) => return false,
    };
    let file_path = match file.path().to_str().and_then(|p| ffi::CString::new(p).ok()) {
        Some(path) => path,
        None => return false,
    };
    let is_executable_by_user =
        unsafe { libc::access(file_path.into_raw(), libc::X_OK) == libc::EXIT_SUCCESS };
    let has_executable_flag = file_metadata.permissions().mode() & EXECUTABLE_FLAGS != 0;
    is_executable_by_user && has_executable_flag && file_metadata.is_file()
}

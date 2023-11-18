use camino::Utf8PathBuf as PathBuf;
use std::fs;

pub(crate) fn read_files(dir: &str) -> Result<Vec<(PathBuf, String)>, String> {
    let files = fs::read_dir(&dir).map_err(|e| e.to_string())?;

    let files = files
        .map(|f| f.map(|f| f.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let contents: Vec<_> = files
        .iter()
        .map(|f| fs::read_to_string(f))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(files
        .into_iter()
        .filter(|f| f.is_file())
        .map(|f| PathBuf::from_path_buf(f).expect("non-utf8 path"))
        .zip(contents)
        .collect())
}

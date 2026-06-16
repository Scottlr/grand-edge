use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use polars::prelude::{DataFrame, ParquetCompression, ParquetReader, ParquetWriter, SerReader};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::AnalyticsError;

pub fn write_parquet(path: &Path, frame: &mut DataFrame) -> Result<u64, AnalyticsError> {
    let writer = BufWriter::new(File::create(path)?);
    ParquetWriter::new(writer)
        .with_compression(ParquetCompression::Zstd(None))
        .finish(frame)?;
    Ok(frame.height() as u64)
}

pub fn parquet_bytes(frame: &mut DataFrame) -> Result<Vec<u8>, AnalyticsError> {
    let temp = NamedTempFile::new()?;
    let path = temp.path().to_path_buf();
    write_parquet(&path, frame)?;
    Ok(std::fs::read(path)?)
}

pub fn read_parquet(path: &Path) -> Result<DataFrame, AnalyticsError> {
    let file = File::open(path)?;
    Ok(ParquetReader::new(file).finish()?)
}

pub fn sha256_file(path: &Path) -> Result<String, AnalyticsError> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

pub fn write_parquet_bytes_to_temp(
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(NamedTempFile, PathBuf), AnalyticsError> {
    let temp = NamedTempFile::new()?;
    std::fs::write(temp.path(), bytes)?;
    Ok((temp, relative_path.to_path_buf()))
}

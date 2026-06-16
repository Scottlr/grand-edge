use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use polars::prelude::{DataFrame, ParquetReader, ParquetWriter, SerReader};
use sha2::{Digest, Sha256};

use crate::AnalyticsError;

pub fn write_parquet(path: &Path, frame: &mut DataFrame) -> Result<u64, AnalyticsError> {
    let writer = BufWriter::new(File::create(path)?);
    ParquetWriter::new(writer).finish(frame)?;
    Ok(frame.height() as u64)
}

pub fn read_parquet(path: &Path) -> Result<DataFrame, AnalyticsError> {
    let file = File::open(path)?;
    Ok(ParquetReader::new(file).finish()?)
}

pub fn sha256_file(path: &Path) -> Result<String, AnalyticsError> {
    let bytes = std::fs::read(path)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

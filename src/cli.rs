use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum MetadataBackend {
    Jsonl,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliOptions {
    /// Input image (raw, E01, or device)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output directory for carved files and metadata
    #[arg(short, long, default_value = "./output")]
    pub output: PathBuf,

    /// Optional path to config file (YAML)
    #[arg(long)]
    pub config_path: Option<PathBuf>,

    /// Enable GPU acceleration if available
    #[arg(long)]
    pub gpu: bool,

    /// Number of worker threads
    #[arg(long, default_value_t = num_cpus::get())]
    pub workers: usize,

    /// Chunk size, in MiB
    #[arg(long, default_value_t = 512)]
    pub chunk_size_mib: u64,

    /// Chunk overlap, in KiB (overrides config when set)
    #[arg(long)]
    pub overlap_kib: Option<u64>,

    /// Metadata backend
    #[arg(long, value_enum, default_value_t = MetadataBackend::Jsonl)]
    pub metadata_backend: MetadataBackend,
}

pub fn parse() -> CliOptions {
    CliOptions::parse()
}

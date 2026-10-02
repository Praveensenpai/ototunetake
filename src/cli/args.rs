use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "ototunetake",
    version,
    about = "A dedicated Japanese Shadowing Player using Anki as the source of truth",
    long_about = "A dedicated Japanese Shadowing Player inspired by Ototune that turns reviewed Anki sentences into a continuous shadowing and listening library."
)]
pub struct CliArgs {
    #[arg(short = 'c', long, help = "Path to Anki collection.anki2 database")]
    pub collection: Option<PathBuf>,

    #[arg(short = 'm', long, help = "Path to Anki collection.media folder")]
    pub media: Option<PathBuf>,

    #[arg(short = 'd', long, help = "Filter sentences by deck name")]
    pub deck: Option<String>,

    #[arg(short = 't', long, help = "Filter sentences by tag")]
    pub tag: Option<String>,

    #[arg(long, help = "Filter only mature reviewed cards (ivl >= 21d)")]
    pub mature: bool,

    #[arg(long, help = "Filter only young reviewed cards (ivl < 21d)")]
    pub young: bool,

    #[arg(
        long,
        help = "Initial preset (listen, repeat, shadow)",
        default_value = "shadow"
    )]
    pub mode: String,

    #[arg(short = 's', long, help = "Start with queue shuffled")]
    pub shuffle: bool,

    #[arg(long, help = "Show Japanese text by default")]
    pub show_text: bool,
}

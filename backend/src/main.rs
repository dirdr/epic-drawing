use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use epic_drawing_backend::config::{ImageConfig, load_config};
use epic_drawing_backend::observer::FileObserver;
use epic_drawing_backend::pipeline::{Pipeline, PipelineConfig};

#[derive(Parser)]
#[command(name = "epic_drawing")]
#[command(about = "Image processing pipeline for epicycloid drawing")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Process {
        #[arg(help = "Image name from images.toml (required unless --all)")]
        image: Option<String>,

        #[arg(
            long,
            conflicts_with = "image",
            help = "Process all images from images.toml"
        )]
        all: bool,

        #[arg(
            short,
            long,
            help = "Enable debug mode - save intermediate pipeline steps"
        )]
        debug: bool,

        #[arg(
            short,
            long,
            default_value = "output",
            help = "Output directory for debug images"
        )]
        output_dir: PathBuf,
    },
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Process {
            image,
            all,
            debug,
            output_dir,
        } => {
            let config = load_config("images.toml").context("Failed to load images.toml")?;

            let images: Vec<&ImageConfig> = if all {
                config.image.iter().collect()
            } else {
                let name = image.context("Specify an image name or use --all")?;
                vec![
                    config
                        .find_image(&name)
                        .with_context(|| format!("Image '{}' not found in images.toml", name))?,
                ]
            };

            for img_config in images {
                process_image(img_config, debug, &output_dir)?;
            }
        }
        Commands::List => {
            let config = load_config("images.toml").context("Failed to load images.toml")?;
            println!("Available images:");
            for img in &config.image {
                println!("  {} - {}", img.name, img.file);
            }
        }
    }

    Ok(())
}

fn process_image(config: &ImageConfig, debug: bool, output_dir: &PathBuf) -> Result<()> {
    println!("Processing: {}", config.name);

    let bytes = std::fs::read(&config.file)
        .with_context(|| format!("Failed to read image file: {}", config.file))?;

    let pipeline_config = PipelineConfig::from_image_config(config);

    let contour = if debug {
        std::fs::create_dir_all(output_dir)?;
        let observer = FileObserver::new(output_dir.clone(), config.name.clone());
        let mut pipeline = Pipeline::with_observer(observer);
        pipeline.process(&bytes, &pipeline_config)?
    } else {
        let mut pipeline = Pipeline::new();
        pipeline.process(&bytes, &pipeline_config)?
    };

    println!("  Contour points: {}", contour.len());
    Ok(())
}

// start a clap cli
use clap::{Parser, Subcommand};
use reqwest::Url;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        /// The name of the hxql app.
        #[arg(short, long)]
        name: String,
    },
    Start {
        /// The URL of the GraphQL server.
        #[arg(short, long)]
        graphql: Option<Url>,

        /// The port to start the server on.
        #[arg(short, long, default_value = "5000")]
        port: u16,

        /// The path to the source files.
        #[arg(short, long, default_value = "./src")]
        src: std::path::PathBuf,

        /// Enable hydration with handlebars.
        #[arg(short, long, default_value = "true")]
        enable_hydrate: bool,
    },
}

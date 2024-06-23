//! # Hxql
//! Hxql is a Rust based server-side rendering engine for web applications. It combines together
//! HTML, HTMX, GraphQL, and Handlebars to provide a seamless experience for developers to build
//! web applications.
//!
//! ## Highlights
//! - **Server-side rendering**: Hxql is a server-side rendering engine that allows you to render your web applications server-side.
//! - **HTMX**: Use htmx or html to fetch data from the server.
//! - **GraphQL**: Hxql supports graphql query parsing and automatic query execution.
//! - **Hydration**: Hxql hydrates the server-side rendered HTML with handlebars templates using the data fetched from the graphql server.
//! - **Recursive Directory Routing**: Serve pages (index.html) from any folder or utilize the
//!     route as a template or content to render the page.
//!     - Example: `src/pages/about/index.html` will be served at `/about`.
//! - **Static Assets**: Serve static assets from the `public` folder.
//!
//! ## Getting Started
//! - Install hxql using `cargo install hxql`.
//! - Initialize a new project using `hxql init <project-name>`.
//! - Start the server using `hxql start`.
//!
//! ## Usage
//!
//! ```shell
//! hxql start --graphql <graphql-url> --port <port> --src <src> --enable-hydrate
//! ```

use clap::Parser;
use log::*;
use warp::http::HeaderMap;
use warp::Filter;

use crate::serve::Serve;

mod cli;
mod serve;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Init { name } => {
            info!("Initializing project");
            info!("Cloning the template project");
            std::process::Command::new("git")
                //TODO: Change the URL to the actual template
                .args(&[
                    "clone",
                    "https://github.com/The-Devoyage/hxql-starter.git",
                    &name,
                ])
                .output()
                .expect("Failed to clone the template project");
            info!("Initializing Git repository");
            // Remove existing git repository
            std::process::Command::new("rm")
                .args(&["-rf", &format!("{}/.git", &name)])
                .output()
                .expect("Failed to remove the existing git repository");
            // Initialize a new git repository
            std::process::Command::new("git")
                .args(&["init", &name])
                .output()
                .expect("Failed to initialize a new git repository");
            info!("Project initialized successfully");
        }
        cli::Commands::Start {
            graphql,
            port,
            src,
            enable_hydrate,
        } => {
            info!("Starting server on port {}", port);
            let ssr_src = src.clone();

            let ssr_route = warp::path::full()
                .and(warp::post().or(warp::get()))
                .and(warp::body::form())
                .and(warp::query::<serde_json::Value>())
                .and(warp::header::headers_cloned())
                .and_then(
                    move |path: warp::filters::path::FullPath,
                          _,
                          body: serde_json::Value,
                          search: serde_json::Value,
                          headers: HeaderMap| {
                        let graphql = graphql.clone();
                        let enable_hydrate = enable_hydrate.clone();
                        let src = ssr_src.clone();
                        async move {
                            let response = Serve::new(
                                path,
                                body,
                                search,
                                graphql,
                                std::path::PathBuf::from(&src.clone()),
                                enable_hydrate,
                                headers,
                            )
                            .build()
                            .await;
                            response
                        }
                    },
                );

            let assets_route = warp::path("public")
                .and(warp::fs::dir(std::path::PathBuf::from(&src).join("public")))
                .and_then(move |file| async move { Ok::<_, warp::Rejection>(file) });

            warp::serve(assets_route.or(ssr_route))
                .run(([127, 0, 0, 1], port))
                .await;
        }
    }
}

use std::fmt::Debug;

use handlebars::{Handlebars, RenderError};
use log::*;
use serde_json::json;
use warp::{http::Response, reject::Rejection};

#[derive(Debug)]
pub struct Serve {
    path: warp::filters::path::FullPath,
    body: serde_json::Value,
    search: serde_json::Value,
    graphql: Option<reqwest::Url>,
    serve_path: std::path::PathBuf,
    enable_hydrate: bool,
    _headers: warp::http::HeaderMap,
    context: Option<serde_json::Value>,
}

pub type FileContents = String;
pub type Extension = String;

impl Serve {
    pub fn new(
        path: warp::filters::path::FullPath,
        body: serde_json::Value,
        search: serde_json::Value,
        graphql: Option<reqwest::Url>,
        serve_path: std::path::PathBuf,
        enable_hydrate: bool,
        headers: warp::http::HeaderMap,
    ) -> Self {
        Serve {
            path,
            body,
            search,
            graphql,
            serve_path,
            enable_hydrate,
            _headers: headers,
            context: None,
        }
    }

    fn handle_hydrate_ssr(&self, html_template: String) -> Result<String, RenderError> {
        let handlebars = Handlebars::new();

        let context = if let Some(context) = &self.context {
            context
        } else {
            &serde_json::Value::Null
        };

        let html = handlebars.render_template(&html_template, context);
        html
    }

    // Recusively search up the path for the index.html file
    fn search_up_path(file_path: String) -> Result<String, Rejection> {
        warn!("Search Up Path: {:?}", file_path);
        let path = file_path;
        let params = path.split("/").collect::<Vec<&str>>();
        for i in 0..params.len() {
            let path = params[0..params.len() - i].join("/");
            let file_path = format!("{}/index.html", path);
            trace!("Implied HTML Extension: {:?}", file_path);
            let file = std::fs::read_to_string(file_path).ok();
            if file.is_some() {
                return Ok(file.unwrap());
            }

            // if all the way up to the root directory, return a 404
            if path == "" {
                return Err(warp::reject::not_found());
            }
        }
        Err(warp::reject::not_found())
    }

    fn get_file(&self) -> Result<(FileContents, Option<Extension>), Rejection> {
        let file_path = format!(
            "{}{}",
            self.serve_path.to_str().unwrap(),
            self.path.as_str()
        );
        let ext;
        let is_directory = std::fs::metadata(file_path.clone())
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false);
        let formatted = if is_directory {
            let file_path = format!("{}/index.html", file_path);
            let file = std::fs::read_to_string(file_path.clone()).ok();
            ext = Some("html".to_string());
            let file = match file {
                Some(file) => file,
                None => {
                    let file = Serve::search_up_path(file_path);
                    match file {
                        Ok(file) => file,
                        Err(err) => {
                            error!("Could not find index.html file: {:?}", err);
                            return Err(warp::reject::not_found());
                        }
                    }
                }
            };
            file
        } else {
            ext = file_path.split(".").last().map(|s| s.to_string());
            let file = std::fs::read_to_string(file_path.clone());
            match file {
                Ok(file) => file,
                Err(err) => {
                    error!("Error reading file: {:?} -- {:?}", file_path, err);
                    return Err(warp::reject::not_found());
                }
            }
        };

        Ok((formatted, ext))
    }

    fn get_query(&self) -> Option<serde_json::Value> {
        if let Some(query) = self.body.get("query") {
            return Some(query.clone());
        } else {
            let value = self.search.get("query");
            match value {
                Some(v) => Some(v.clone()),
                None => None,
            }
        }
    }

    fn get_operation_name(&self) -> Option<serde_json::Value> {
        if let Some(operation_name) = self.body.get("operation_name") {
            return Some(operation_name.clone());
        } else {
            let value = self.search.get("operation_name");
            match value {
                Some(v) => Some(v.clone()),
                None => None,
            }
        }
    }

    fn get_variables(&self) -> Option<serde_json::Value> {
        if let Some(variables) = self.body.get("variables") {
            let v = serde_json::from_str(variables.as_str().unwrap());
            match v {
                Ok(v) => Some(v),
                Err(err) => {
                    error!("Failed to parse variables: {:?}", err);
                    None
                }
            }
        } else {
            let value = self.search.get("variables");
            match value {
                Some(v) => {
                    //Convert v from string to object
                    let v = serde_json::from_str(v.as_str().unwrap());
                    match v {
                        Ok(v) => Some(v),
                        Err(err) => {
                            error!("Failed to parse variables: {:?}", err);
                            None
                        }
                    }
                }
                None => None,
            }
        }
    }

    fn get_props(&self) -> Option<serde_json::Value> {
        if let Some(props) = self.body.get("props") {
            let v = serde_json::from_str(props.as_str().unwrap());
            match v {
                Ok(v) => Some(v),
                Err(err) => {
                    error!(
                        "Failed to parse props: {:?}. Ensure props is of type json object.",
                        err
                    );
                    None
                }
            }
        } else {
            let value = self.search.get("props");
            match value {
                Some(v) => {
                    //Convert v from string to object
                    let v = serde_json::from_str(v.as_str().unwrap());
                    match v {
                        Ok(v) => Some(v),
                        Err(err) => {
                            error!(
                                "Failed to parse props: {:?}. Ensure props is of type json object.",
                                err
                            );
                            None
                        }
                    }
                }
                None => None,
            }
        }
    }

    pub fn validate_query(&self) -> Result<(), Rejection> {
        let query = self.get_query();
        let operation_name = self.get_operation_name();
        let variables = self.get_variables();

        if query.is_none() && operation_name.is_none() && variables.is_none() {
            return Ok(());
        }

        // Validate the query, operation_name, and variables exist
        if query.is_none() {
            error!("SSR Hydrate Error: Query is required");
            return Err(warp::reject::not_found());
        }
        if operation_name.is_none() {
            error!("SSR Hydrate Error: Operation Name is required");
            return Err(warp::reject::not_found());
        }
        if variables.is_none() {
            error!("SSR Hydrate Error: Variables is required");
            return Err(warp::reject::not_found());
        }

        Ok(())
    }

    pub fn serve_static(&self) -> Result<impl warp::Reply, warp::Rejection> {
        let (file_contents, ext) = self.get_file()?;

        // Serve Static Files
        if let Some(ext) = ext.clone() {
            let content_type = match ext.as_str() {
                "css" => "text/css",
                "js" => "application/javascript",
                "json" => "application/json",
                "png" => "image/png",
                "jpg" => "image/jpeg",
                "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "svg" => "image/svg+xml",
                "ico" => "image/x-icon",
                "txt" => "text/plain",
                _ => return Ok(Response::builder().body(file_contents)),
            };
            let response = Response::builder()
                .header("Content-Type", content_type)
                .body(file_contents);
            return Ok(response);
        };

        Ok(Response::builder().body(file_contents))
    }

    async fn populate_graphql_context(&mut self) -> Result<(), Rejection> {
        let query = self.get_query();
        let operation_name = self.get_operation_name();
        let variables = self.get_variables();
        let graphql_url = self.graphql.clone();

        if query.is_some()
            && operation_name.is_some()
            && variables.is_some()
            && graphql_url.is_some()
        {
            let graphql_request = json!({
                "query": query.unwrap(),
                "operationName": operation_name.unwrap(),
                "variables": variables.unwrap(),
            });
            let client = reqwest::Client::new();
            println!("GRAPHQL URL: {:?}", graphql_url);
            println!("GRAPHQL REQUEST: {:?}", graphql_request);
            let request = client
                .post(graphql_url.unwrap())
                .json(&graphql_request)
                .header("Content-Type", "application/json")
                .send()
                .await
                .map_err(|err| {
                    error!("Failed to send GraphQL Request: {:?}", err);
                    warp::reject::not_found() //TODO: Return a proper error
                })?;
            println!("REQUEST: {:?}", request);
            let response = request.json::<serde_json::Value>().await.map_err(|err| {
                error!("Failed to parse GraphQL Response: {:?}", err);
                warp::reject::not_found() //TODO: Return a proper error
            })?;

            if response.get("errors").is_some() {
                error!("GraphQL Error: {:?}", response);
                return Err(warp::reject::not_found()); //TODO: Return a proper error
            }

            // context = Some(response.get("data").unwrap().clone());
            self.context = Some(response.get("data").unwrap().clone());

            return Ok(());
        }

        Ok(())
    }

    fn populate_props_context(&mut self) -> Result<(), Rejection> {
        let props = self.get_props();

        if props.is_some() {
            let props = props.unwrap();

            if props.as_object().is_none() {
                error!("SSR Hydrate Error: Props must be an object");
                return Err(warp::reject::not_found());
            }

            self.context = Some(props);

            return Ok(());
        }

        Ok(())
    }

    pub async fn build(&mut self) -> Result<impl warp::Reply, warp::Rejection> {
        self.serve_static()?;
        let (file_contents, ext) = self.get_file()?;

        if self.enable_hydrate && ext == Some("html".to_string()) {
            self.validate_query()?;
            self.populate_graphql_context().await?;
            self.populate_props_context()?;

            let html = self.handle_hydrate_ssr(file_contents);
            match html {
                Ok(html) => Ok(Response::builder()
                    .header("Content-Type", "text/html")
                    .body(html)),
                Err(err) => {
                    error!("SSR Hydrate Error: {:?}", err);
                    return Err(warp::reject::not_found());
                }
            }
        } else if self.enable_hydrate && ext != Some("html".to_string()) {
            return Ok(Response::builder().body(file_contents));
        } else {
            Ok(Response::builder().body(file_contents))
        }
    }
}

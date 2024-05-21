# hxql

Start and run a dynamic web app with this cli. Centered around htmx, powered by
GraphQL, and driven by @the-devoyage/subgraph, this cli can be used to start, run, and 
interface with an hxql app.

Currently this app is in the early planniing stages.

## Hxql Manifesto

Hxql is a web framework and design pattern implementation. It is built around GraphQL and
Htmx, an unlikely duo that work together when powerd by Subgraph - a tool to create dynamic
web APIs.

Htmx is commonly associated with static or server side rendered apps which serve up pre-rendered
html content. On the other hand, apps centered around GraphQL typically receive serialized 
data such as JSON which is injected into "templates" client side. 

For this reason, utilizing these two technologies together becomes a anti-pattern. 

Hxql ties together modern and familair technologies to create a seamless development experience
based on this anti-pattern.

- HTML First
    - Write in web's native markup language.
- Recursive Directory Routing
    - Routing is as easy as making a folder with an `index.html`. 
- Componentized Development
    - Serve html components and dynamically inject them into the DOM using HTMX.
- Server Side Rendered Templates
    - Handblebars enabled and Instant Data Access.
    - No clients to fetch data. Write a graphql query and provide variables and the data
    becomes instantly available to the template.
- Entity Driven API
    - Public facing, typesafe, and logical API. Built around the entities it serves, rather
    than the html it hydrates.

## Getting Started

### Install

`cargo install hxql`

### Intitalize 

`hxql init`

Creates a directory with a ready to run web application.

### Run

`hxql start`

Reads a config file, `hxql.toml` in the same directory, and uses docker (subgraph) to run the
web application.

## Config

Creates a config file that serves dual purpose for both the hxql CLI and the Subgraph API.

## API

- `init` - Clones the hxql starter into a custom named folder and a base subgraph configuration.
- `run` - Reads the `hxql.toml` file and uses docker (subgraph) to start the web server.
- `datasource`
    - `--add` - Adds a new datasource.
    - `--remove` - Removes a datasource.
    - `--modify` - Updates a datasource.
- `entity`
    - `--add` - Adds a new entity.
    - `--remove` - Removes a entity.
    - `--modify` - Updates a entity.
- `guard`
    - `--add` - Adds a new guard to the subgraph config.
    - `--remove` - Removes a guard from the subgraph config.
    - `--modify` - Updates a guard.

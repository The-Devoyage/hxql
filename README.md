# hxql

Start and run a dynamic web app with this cli. Centered around htmx, paired with
GraphQL, and built with rust - hxql is a cli that can be used to start, run, and 
interface with an hxql app.

Currently this app is in the early planning stages. For examples, check out the
starter repo - https://github.com/The-Devoyage/hxql-starter

## Hxql Manifesto

Hxql is a web framework and design pattern implementation. It is built around GraphQL and
Htmx, an unlikely duo that can come together to create a delightfuly simple and 
versatile, and dynamic development experience. 

Htmx is commonly associated with static or server side rendered apps which serve up pre-rendered
html content. On the other hand, apps centered around GraphQL typically receive serialized 
data such as JSON which is injected into "templates" or components client side. 

For this reason, utilizing these two technologies together becomes a anti-pattern. Delivering
pre-renndered content kind of contridicts the need for templated variable injection client side.

Hxql ties together modern and familair technologies to create a seamless development experience
based on this "anti-pattern".

## Hxql Features

- HTML First
    - Write in web's native markup language.
- Recursive Directory Routing
    - Routing is as easy as making a folder with an `index.html`. 
- Componentized Development
    - Serve html components and dynamically inject them into the DOM (using HTMX for example).
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

`hxql start <options>`

## API

- `init` - Clones the hxql starter into a custom named folder and a base subgraph configuration.
    - `--name` - Specify a name for your hxql app.
- `start` - Reads the `hxql.toml` file and uses docker (subgraph) to start the web server.
    - `--port` - Specifies a port to run the server on.
    - `--host` - Binds the port to all network interfaces.
    - `--graphql` - Point to the GraphQL Server providing hydration for the hxql app.
    - `--enable-hydrate` - Toggle hydration settings. On by default.

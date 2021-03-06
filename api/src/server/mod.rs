//! All code related to the webserver. Basically anything that calls Actix
//! lives here.

mod websocket;

use crate::{
    gql::{self, GqlContext, GqlSchema},
    util::Pool,
};
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use std::{io, sync::Arc};

#[get("/graphiql")]
pub async fn route_graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/graphql")]
pub async fn route_graphql(
    pool: web::Data<Pool>,
    st: web::Data<Arc<GqlSchema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = web::block(move || {
        let res = data.execute(
            &st,
            &GqlContext {
                pool: pool.into_inner(),
            },
        );
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

#[actix_rt::main]
pub async fn run_server(pool: Pool, host: String) -> io::Result<()> {
    // Set up logging
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    // Init GraphQL schema
    let gql_schema = Arc::new(gql::create_gql_schema());

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Need to clone these because init occurs once per thread
            .data(pool.clone())
            .data(gql_schema.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // routes
            .service(route_graphql)
            .service(route_graphiql)
            .service(websocket::ws_program_specs_by_slugs)
    })
    .bind(host)?
    .run()
    .await
}

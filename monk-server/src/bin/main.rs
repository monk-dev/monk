use actix_cors::Cors;

use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Result};
use async_graphql::{
    extensions::Tracing,
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptyMutation, EmptySubscription, Schema,
};
use async_graphql_actix_web::{Request, Response};
use monk_db::{graphql::MonkSchema, init_db, mutation::Mutation, query::Query};
use tracing::info;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

#[post("/graphql")]
async fn graphql(schema: web::Data<MonkSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

#[get("/graphql/playground")]
async fn graphql_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql"),
        )))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .init();

    let db_conn = init_db("monk.db")?;
    info!("DB Conn: {:?}", db_conn);

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db_conn)
        .extension(Tracing)
        .finish();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_origin()
            .allow_any_header();

        App::new()
            .data(schema.clone())
            .service(graphql)
            .service(graphql_playground)
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind("127.0.0.1:5555")?
    .run()
    .await
    .map_err(Into::into)
}

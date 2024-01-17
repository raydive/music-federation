use std::net::SocketAddr;

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use tokio::net::TcpListener;

#[derive(SimpleObject, Debug, Clone)]
struct Album {
    id: ID,
    title: String,
    year: i32,
}

struct Query;

#[Object]
impl Query {
    async fn albums(&self, ctx: &Context<'_>) -> Vec<Album> {
        let data = ctx.data_unchecked::<Vec<Album>>();
        data.to_vec()
    }

    async fn find_album(&self, ctx: &Context<'_>, id: ID) -> Option<Album> {
        let data = ctx.data_unchecked::<Vec<Album>>();
        data.iter().find(|m| m.id == id).cloned()
    }

    // Apollo Federationで使用する
    #[graphql(entity)]
    async fn find_album_entity_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Album> {
        let data = ctx.data_unchecked::<Vec<Album>>();
        data.iter().find(|m| m.id == id).cloned()
    }
}

async fn graphql_handler(
    State(schema): State<Schema<Query, EmptyMutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, Response> {
    let response = schema.execute(req.into_inner()).await;
    match response.into_result() {
        Ok(res) => Ok(res.into()),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()),
    }
}

#[tokio::main]
async fn main() {
    let albums = vec![
        Album {
            id: "1".into(),
            title: "album1".into(),
            year: 2020,
        },
        Album {
            id: "2".into(),
            title: "album2".into(),
            year: 2021,
        },
    ];

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(albums)
        .finish();

    let app = Router::new()
        .route("/", post(graphql_handler))
        .with_state(schema);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listner = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listner, app.into_make_service())
        .await
        .unwrap();
}

use std::net::SocketAddr;

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router
};
use tokio::net::TcpListener;

#[derive(SimpleObject, Debug, Clone)]
struct Album {
    id: ID,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(shareable)]
struct Musician {
    id: ID,
    name: String,
    age: i32,
    albums: Vec<Album>,
}

struct Query;

#[Object]
impl Query {
    async fn musicians(&self, ctx: &Context<'_>) -> Vec<Musician> {
        let data = ctx.data_unchecked::<Vec<Musician>>();
        data.to_vec()
    }

    async fn find_musician(&self, ctx: &Context<'_>, id: ID) -> Option<Musician> {
        let data = ctx.data_unchecked::<Vec<Musician>>();
        data.iter().find(|m| m.id == id).cloned()
    }

    // Apollo Federationで使用する
    // 例えば別のサブグラフでmusicianの情報が必要な場合、idを用いたresolverになる
    #[graphql(entity)]
    async fn find_musician_entity_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<Musician> {
        let data = ctx.data_unchecked::<Vec<Musician>>();
        data.iter().find(|m| m.id == id).cloned()
    }

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
        },
        Album {
            id: "2".into(),
        },
    ];
    let data = vec![
        Musician {
            id: "1".into(),
            name: "John".to_string(),
            age: 20,
            albums: albums.clone(),
        },
        Musician {
            id: "2".into(),
            name: "Paul".to_string(),
            age: 22,
            albums: albums.clone(),
        },
        Musician {
            id: "3".into(),
            name: "George".to_string(),
            age: 24,
            albums: albums.clone(),
        },
        Musician {
            id: "4".into(),
            name: "Ringo".to_string(),
            age: 26,
            albums: albums.clone(),
        },
    ];

    let schema = async_graphql::Schema::build(
        Query,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .data(data)
    .data(albums)
    .finish();
    let app = Router::new()
        .route("/", post(graphql_handler))
        .with_state(schema);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listner = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listner, app.into_make_service())
        .await
        .unwrap();
}

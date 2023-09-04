use std::net::SocketAddr;

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(shareable)]
struct Instrument {
    id: ID,
    name: String,
}

struct Query;

#[Object]
impl Query {
    async fn instruments(&self, ctx: &Context<'_>) -> Vec<Instrument> {
        let data = ctx.data_unchecked::<Vec<Instrument>>();
        data.to_vec()
    }

    #[graphql(entity)]
    async fn find_instrument(&self, ctx: &Context<'_>, id: ID) -> Option<Instrument> {
        let data = ctx.data_unchecked::<Vec<Instrument>>();
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
    let data = vec![
        Instrument {
            id: "1".into(),
            name: "Guitar".to_string(),
        },
        Instrument {
            id: "2".into(),
            name: "Bass".to_string(),
        },
        Instrument {
            id: "3".into(),
            name: "Drums".to_string(),
        },
    ];

    let schema = async_graphql::Schema::build(
        Query,
        async_graphql::EmptyMutation,
        async_graphql::EmptySubscription,
    )
    .data(data)
    .finish();
    let app = Router::new()
        .route("/", post(graphql_handler))
        .with_state(schema);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

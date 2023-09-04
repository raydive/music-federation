use std::net::SocketAddr;

use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};

#[derive(SimpleObject, Debug, Clone)]
struct Album {
    id: ID,
    title: String,
    musicians: Vec<Musician>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(shareable)]
struct Musician {
    id: ID,
    instruments: Vec<Instrument>,
}

#[ComplexObject]
impl Musician {
    async fn main_instruments(&self) -> Instrument {
        self.instruments[0].clone()
    }
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(shareable)]
struct Instrument {
    id: ID,
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

struct Data {
    albums: Vec<Album>,
    musicians: Vec<Musician>,
}

#[tokio::main]
async fn main() {
    let data = vec![
        Album {
            id: "1".into(),
            title: "album1".into(),
            musicians: vec![
                Musician {
                    id: "1".into(),
                    instruments: vec![Instrument { id: "1".into() }, Instrument { id: "2".into() }],
                },
                Musician {
                    id: "2".into(),
                    instruments: vec![Instrument { id: "3".into() }],
                },
            ],
        },
        Album {
            id: "2".into(),
            title: "album2".into(),
            musicians: vec![
                Musician {
                    id: "1".into(),
                    instruments: vec![Instrument { id: "3".into() }, Instrument { id: "2".into() }],
                },
                Musician {
                    id: "3".into(),
                    instruments: vec![Instrument { id: "1".into() }],
                },
            ],
        },
    ];

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(data)
        .finish();

    let app = Router::new()
        .route("/", post(graphql_handler))
        .with_state(schema);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use async_sqlite::{Client, ClientBuilder, JournalMode};
use axum::{extract::State, response::Html, routing::get, Form, Json, Router};
use leptos::{ssr::render_to_string, *};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

#[derive(Clone)]
struct AppState {
    sqlite: Client,
}

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new()
        .path("/Users/fireland/Documents/code/rust_yeti/test.db")
        .journal_mode(JournalMode::Wal)
        .open()
        .await
        .unwrap();

    client
        .conn(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS test_data (name TEXT, date TEXT)",
                [],
            )
        })
        .await
        .unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .compact()
        .init();

    let app_state = AppState { sqlite: client };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(get_index))
        .route("/data", get(get_data).post(post_data))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[component]
fn NewData() -> impl IntoView {
    view! {
        <form hx-post="/data" hx-target="this" hx-swap="outerHTML">
            <div>
                <label>"Name"</label>
                <input name="name"/>
            </div>
            <div>
                <label>"Date"</label>
                <input name="date"/>
            </div>
            <div>
                <label>"Tags"</label>
                <input name="tags"/>
            </div>
            <button type="submit">"Save"</button>
        </form>
    }
}

async fn get_index() -> Html<String> {
    Html(
        render_to_string(|| {
            view! {
                <html>
                    <head>
                        <script src="https://unpkg.com/htmx.org@1.9.6"></script>
                    </head>
                    <body>
                        <h1>Hello World</h1>
                        <button hx-get="/data" hx-swap="outerHTML">
                            "Click Me"
                        </button>
                        <NewData />
                    </body>
                </html>
            }
        })
        .to_string(),
    )
}

async fn get_data(State(app_state): State<AppState>) -> Html<String> {
    let my_data = app_state
        .sqlite
        .conn(|conn| {
            let mut statement = conn.prepare("SELECT * FROM test_data").unwrap();
            let mut rows = statement.query([]).unwrap();
            let mut names = Vec::new();
            while let Some(row) = rows.next().unwrap() {
                names.push(MyData {
                    name: row.get("name").unwrap(),
                    date: row.get("date").unwrap(),
                    tags: vec![],
                });
            }
            Ok(names)
        })
        .await
        .unwrap();

    Html(
        render_to_string(|| {
            view! {
                <ul>
                    {my_data.into_iter()
                        .map(|item| view! {<li>{item.name}</li>})
                        .collect_view()}
                </ul>
            }
        })
        .to_string(),
    )
}

const DATA_TEMPLATE: &'static str = r#"
<ul>
    {% for item in data %}
    <li>{{item.name}} ({{item.date}})</li>
    {% endfor %}
</ul>
"#;

#[derive(Deserialize)]
struct CreateMyData {
    name: String,
    date: String,
    tags: String,
}

async fn post_data(
    State(app_state): State<AppState>,
    Form(payload): Form<CreateMyData>,
) -> Json<Value> {
    app_state
        .sqlite
        .conn(|conn| {
            conn.execute(
                "INSERT INTO test_data (name, date) VALUES (?1, ?2)",
                [payload.name, payload.date],
            )
        })
        .await
        .unwrap();
    Json(json!(MyData {
        name: "".to_string(),
        date: String::from(""),
        tags: Vec::new(),
    }))
}

#[derive(Debug, Serialize)]
struct MyData {
    name: String,
    date: String,
    tags: Vec<String>,
}

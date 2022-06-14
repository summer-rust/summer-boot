use serde::Deserialize;
use summer_boot::{Request, Result};

#[derive(Debug, Deserialize)]
struct User {
    name: String,
    age: u16,
}

#[summer_boot::main]
async fn main() {

    let mut app = summer_boot::new();
    app.at("/test/api").post(test_api);
    app.listen("127.0.0.1:8080").await.unwrap();
}

async fn test_api(mut req: Request<()>) -> Result {
    let User { name, age } = req.body_json().await?;
    Ok(format!("Hello, {}!  {} years old", name, age).into())
}
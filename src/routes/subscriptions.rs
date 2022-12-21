use actix_web::{web, App, HttpResponse, HttpServer, guard::Connect};
use sqlx::{PgConnection, PgPool};
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>, // renamed
    // OLD VERSION w/ PG Connection
    // Retrieving a connection from the application state! 
    // connection: web::Data<PgConnection>,
) -> HttpResponse {
    // `Result` has two variants: `Ok` and `Err`.
    // The first for successes, the second for failures.
    // We use a `match` statement to choose what to do based on the outcome.
    // We will talk more about `Result` going forward!
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    // We use "get_ref" to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.

    // Using the pool as a drop-in replacement
    .execute(pool.get_ref())
    .await{
        Ok(_) => HttpResponse::Ok().finish(), 
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish() 
        }
    }

}
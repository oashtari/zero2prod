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
    // Let's generate a random unique identifier 
    let request_id = Uuid::new_v4();
    tracing::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id, 
        form.email,
        form.name
    );

    tracing::info!("request_id {} - Saving new subscriber details in the database", request_id);
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
        Ok(_) => {
            tracing::info!("request_id {} - New subscriber details have been saved", request_id);
            HttpResponse::Ok().finish()
        }, 
        Err(e) => {
            // converting print statement into an error log...and now tracing
            tracing::error!("request_id {} - Failed to execute query: {:?}", request_id, e);
            // // Using `println!` to capture information about the error in case things don't work out as expected
            // println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish() 
        }
    }

}
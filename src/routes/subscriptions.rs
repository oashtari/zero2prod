use actix_web::{web, App, HttpResponse, HttpServer, guard::Connect};
use sqlx::{PgConnection, PgPool};
use chrono::Utc;
use uuid::Uuid;
use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        // request_id = %Uuid::new_v4(), // eliminated due to double request IDs showing up
        subscriber_email = %form.email, 
        subscriber_name = %form.name
    )
)]

pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>, // renamed
    // OLD VERSION w/ PG Connection
    // Retrieving a connection from the application state! 
    // connection: web::Data<PgConnection>,
) -> HttpResponse {

    match insert_subscriber(&pool, &form).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish() }
    }
#[tracing::instrument(
    name = "Saving new subscriber details in the database", 
    skip(form, pool)
    )]
    
    // // ADDED TRACING INSTRUMENT TO GET RID OF QUERY SPAN
    // // Let's generate a random unique identifier 
    // let request_id = Uuid::new_v4();

    // // Spans, like logs, have an associated level 
    // // `info_span` creates a span at the info-level 
    // let request_span = tracing::info_span!(
    //     "Adding a new subscriber.",
    //     %request_id,
    //     subscriber_email = %form.email,
    //     subscriber_name = %form.name
    // );
    // // Using `enter` in an async function is a recipe for disaster!
    // // Bear with me for now, but don't do this at home.
    // // See the following section on `Instrumenting Futures`
    // let _request_span_guard = request_span.enter();

    // // `_request_span_guard` is dropped at the end of `subscribe`
    // // That's when we "exit" the span

    // // replacing the interpolation, tracing allows us to associate structured information to our spans as a collection of key-value pairs2.
    // // tracing::info!(
    // //     "request_id {} - Adding '{}' '{}' as a new subscriber.",
    // //     request_id, 
    // //     form.email,
    // //     form.name
    // // );

    // tracing::info!("request_id {} - Saving new subscriber details in the database", request_id);
    // // `Result` has two variants: `Ok` and `Err`.
    // // The first for successes, the second for failures.
    // // We use a `match` statement to choose what to do based on the outcome.
    // // We will talk more about `Result` going forward!

    //  // We do not call `.enter` on query_span!
    // // `.instrument` takes care of it at the right moments 
    // // in the query future lifetime
    // let query_span = tracing::info_span!(
    //     "Saving new subscriber details in the database"
    // );

    pub async fn insert_subscriber(
        pool: &PgPool,
        form: &FormData,
    ) -> Result<(), sqlx::Error> {

    // match -- ELIMINATING THIS AFTER ALL 
        sqlx::query!(
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
    .execute(pool)
    // First we attach the instrumentation, then we `.await` it
    // .instrument(query_span) // error introduced in chapter 5
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error // We will talk about error handling in depth later! 
    })?;
    Ok(())
    // ELIMINATE from await call AFTER ADDING TRACING INSTRUMENT
    // {
    //     Ok(_) => {
    //         tracing::info!("request_id {} - New subscriber details have been saved", request_id);
    //         HttpResponse::Ok().finish()
    //     }, 
    //     Err(e) => {
    //         // Yes, this error log falls outside of `query_span` 
    //         // We'll rectify it later, pinky swear!

    //         // converting print statement into an error log...and now tracing
    //         tracing::error!("request_id {} - Failed to execute query: {:?}", request_id, e);
    //         // // Using `println!` to capture information about the error in case things don't work out as expected
    //         // println!("Failed to execute query: {}", e);
    //         HttpResponse::InternalServerError().finish() 
    //     }
    // }

}
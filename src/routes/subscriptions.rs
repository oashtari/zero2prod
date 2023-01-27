use actix_web::{web, HttpResponse};
use sqlx::{ PgPool};
use chrono::Utc;
use uuid::Uuid;
// use tracing::{Instrument, Subscriber};
use crate::domain::{SubscriberName, NewSubscriber, SubscriberEmail};
// use crate::

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self{email, name})
    }
}

// REPLACED BY TRY FROM METHOD
// pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
//     let name = SubscriberName::parse(form.name)?;
//     let email = SubscriberEmail::parse(form.email)?;
//     Ok(NewSubscriber { email, name })
// }

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
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

    // THIS IS NOW ALL DONE ABOVE IN THE parse_subscriber HELPER FUNCTION
    // let name = match SubscriberName::parse(form.0.name) {
    //     Ok(name) => name,
    //     Err(_) => return HttpResponse::BadRequest().finish(),
    // };

    // let email: SubscriberEmail = match SubscriberEmail::parse(form.0.email) {
    //     Ok(email) => email,
    //     Err(_) => return HttpResponse::BadRequest().finish(),
    // };

    // BEFORE ADDING TYPE DRIVEN DEVELOPMENT
    // let subscriber_name = crate::domain::SubscriberName(form.name.clone());

    // `web::Form` is a wrapper around `FormData`
    // `form.0` gives us access to the underlying `FormData`
   
    // BEFORE THE HELPER FUNCTION
    // let new_subscriber = NewSubscriber { email, name
    //     // we are declaring 'name' above now instead if inline : SubscriberName::parse(form.0.name).expect("Name validation failed."), 
    // };

    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&pool, &new_subscriber).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish() 
    }
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database", 
    skip(new_subscriber, pool)
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
        // form: &FormData,
        new_subscriber: &NewSubscriber,
    ) -> Result<(), sqlx::Error> {

    // match -- ELIMINATING THIS AFTER ALL 
        sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
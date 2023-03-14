use actix_web::{web, HttpResponse, App};
use sqlx::{ PgPool};
use chrono::Utc;
use uuid::Uuid;
// use tracing::{Instrument, Subscriber};
use crate::domain::{SubscriberName, NewSubscriber, SubscriberEmail};
use crate::email_client::{EmailClient, self};
use crate::startup::ApplicationBaseUrl;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Postgres, Transaction};

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

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
    skip(form, pool, email_client, base_url),
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
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>
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

    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

        // // BEFORE ADDING PSEUDO EMAIL CLIENT
        // // match insert_subscriber(&pool, &new_subscriber).await
        // // {
        // //     Ok(_) => HttpResponse::Ok().finish(),
        // //     Err(_) => HttpResponse::InternalServerError().finish() 
        // // }

        // if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        //     return HttpResponse::InternalServerError().finish();
        // }


    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    let subscription_token = generate_subscription_token();
    if store_token(&mut transaction, subscriber_id, &subscription_token).await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber, &base_url.0, &subscription_token).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()
}



#[tracing::instrument(
    name = "Store subscription token in the database", 
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}",e);
        e
    })?;
    Ok(())
}
#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber", 
    skip(email_client, new_subscriber, base_url)
)]

pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    // Build a confirmation link with a dynamic root
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}", base_url, subscription_token);
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome", &html_body, &plain_body)
        .await
}


    // ADDED TO ABOVE
    // Send a (useless) email to the new subscriber.
    // We are ignoring email delivery errors for now.
        // email_client
        //     .send_email(
        //         new_subscriber.email, 
        //         "Welcome", 
        //         &format!(
        //             "Welcome to our newsletter!<br />\
        //             Click <a href=\"{}\">here</a> to confirm your subscription.",
        //             confirmation_link
        //         ), 
        //         &format!(
        //             "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        //             confirmation_link
        //         ),
        //     )
        //     .await
        //     .is_err() {
        //         return HttpResponse::InternalServerError().finish();
        //     }




#[tracing::instrument(
    name = "Saving new subscriber details in the database", 
    skip(new_subscriber, transaction)
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
        transaction: &mut Transaction<'_, Postgres>,
        // form: &FormData,
        new_subscriber: &NewSubscriber,
    ) -> Result<Uuid, sqlx::Error> {

    let subscriber_id = Uuid::new_v4();

    // match -- ELIMINATING THIS AFTER ALL 
        sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')"#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    
    // We use "get_ref" to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.

    // Using the pool as a drop-in replacement
    .execute(transaction)
    // First we attach the instrumentation, then we `.await` it
    // .instrument(query_span) // error introduced in chapter 5
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error // We will talk about error handling in depth later! 
    })?;
    Ok(subscriber_id)
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
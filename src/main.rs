use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse};
use std::fmt::format;
use std::net::TcpListener;
use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use sqlx::{Connection, PgConnection, PgPool, postgres::PgPoolOptions};
use env_logger::Env;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer}; 
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use secrecy::ExposeSecret;

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // // MOVED out to impl, whic his in telemetry.rs now
    // // Redirect all `log`'s events to our subscriber
    // LogTracer::init().expect("Failed to set logger");

    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // `init` does call `set_logger`, so this is all we need to do.
    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set. 

    // We removed the `env_logger` line we had before! -- using tracing now instead
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // // We are falling back to printing all spans at info-level or above 
    // // if the RUST_LOG environment variable has not been set.
    // let env_filter = EnvFilter::try_from_default_env()
    //     .unwrap_or_else(|_| EnvFilter::new("info")); 
    // let formatting_layer = BunyanFormattingLayer::new(
    //     "zero2prod".into(),
    //     // Output the formatted spans to stdout. 
    //     std::io::stdout
    // );

    // // The `with` method is provided by `SubscriberExt`, an extension 
    // // trait for `Subscriber` exposed by `tracing_subscriber`
    // let subscriber = Registry::default()
    //     .with(env_filter)
    //     .with(JsonStorageLayer)
    //     .with(formatting_layer);
    // // `set_global_default` can be used by applications to specify
    // // what subscriber should be used to process spans.
    // set_global_default(subscriber).expect("Failed to set subscriber");


    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration."); 

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
        // No longer async, given that we don't actually try to connect!
        // .await
        // .expect("Failed to create Postgres connection pool.");

    // OLD VERSION w/ PG Connection
    // let connection = PgConnection::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");

    // We have removed the hard-coded `8000` - it's now coming from our settings!
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
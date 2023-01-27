use actix_web::{web, App, HttpServer};
use actix_web::dev::Server;
use actix_web::web::Data;
// use core::time;
// use std::async_iter;
// use core::time;
// use actix_web::middleware::Logger;
use std::net::TcpListener;
use crate::routes::{health_check, subscribe};
use sqlx::{PgPool};
use tracing_actix_web::TracingLogger;
use crate::email_client::{EmailClient};
use crate::configuration::{Settings, DatabaseSettings};
use sqlx::postgres::PgPoolOptions;

                // MOVED INSIDE impl Application
                // pub async fn build(configuration: &Settings) -> Result<Server, std::io::Error> {
                    
                //     // let connection_pool = (PgPoolOptions::new()
                //     //         .acquire_timeout(std::time::Duration::from_secs(2)))
                //     //         .connect_lazy_with(configuration.database.with_db());

                //     let connection_pool = get_connection_pool(&configuration.database);

                //     let sender_email = configuration
                //         .email_client
                //         .sender()
                //         .expect("Invalid sender email address.");

                //     let timeout = configuration.email_client.timeout();

                //     let email_client = EmailClient::new(
                //         configuration.email_client.base_url,
                //         sender_email,
                //         configuration.email_client.authorization_token,
                //         timeout,
                //     );

                //     let address = format!(
                //         "{}:{}",
                //         configuration.application.host,
                //         configuration.application.port
                //     );

                //     let listener = TcpListener::bind(address)?;

                //     run(listener, connection_pool, email_client)
                // }

pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {

    // Wrap the pool using web::Data, which boils down to an Arc smart pointer 
    let db_pool = web::Data::new(db_pool);

    let email_client = Data::new(email_client);

    // OLD VERSION w/ PG connection
    // Wrap the connection in a smart pointer
    // let connection = web::Data::new(connection);
    // capture connection from the surrounding environment
    let server = HttpServer::new( move || {
        App::new()
            // Instead of `Logger::default`
            .wrap(TracingLogger::default())
            // // Middlewares are added using the `wrap` method on `App`
            // .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Register the connection as part of the application state 
            // Get a pointer copy and attach it to the application state
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    
    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // We have converted the `build` function into a constructor for `Application`.

    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host,
            configuration.application.port,
        );

        let listener = TcpListener::bind(&address)?;

        let port = listener.local_addr().unwrap().port();

        let server = run(listener,connection_pool,email_client)?;

        // We "save" the bound port in one of `Application`'s fields
        Ok(Self{port, server})
        
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
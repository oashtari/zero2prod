use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
use crate::routes::{health_check, subscribe};
use sqlx::{PgConnection, PgPool};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {

    // Wrap the pool using web::Data, which boils down to an Arc smart pointer 
    let db_pool = web::Data::new(db_pool);

    // OLD VERSION w/ PG connection
    // Wrap the connection in a smart pointer
    // let connection = web::Data::new(connection);
    // capture connection from the surrounding environment
    let server = HttpServer::new( move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // Register the connection as part of the application state 
            // Get a pointer copy and attach it to the application state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    
    Ok(server)
}

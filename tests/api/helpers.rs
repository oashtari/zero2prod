use actix_web::App;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::{get_connection_pool, Application};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the 
    // value TEST_LOG` because the sink is part of the type returned by
    // `get_subscriber`, therefore they are not the same type. We could work around 
    // it, but this is the most straight-forward way of moving forward.

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name,default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name,default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        };
    // OLD VERSION
    // let subscriber = get_subscriber("test".into(), "debug".into());
});

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}


// At a high-level, we have the following phases:
// • Executetest-specificsetup(i.e.initialiseatracingsubscriber);
// • Randomise the configuration to ensure tests do not interfere with each other (i.e. a different logical
// database for each test case);
// • Initialise external resources(e.g.create and migrate the database!);
// • Build the application;
// • Launch the application as a background task and return a set of resources to interact with it.


pub async fn spawn_app() -> TestApp {

    // The first time `initialize` is invoked the code in `TRACING` is executed. 
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Randomize configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        // Use different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        
        // Use a random OS port
        c.application.port = 0;
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Launch the application as a background task
    // let server = build(configuration.clone()).await.expect("Failed to build application.");

                // // MOVED ABOVE WITH static TRACING
                // // let subscriber = get_subscriber("test".into(), "debug".into());
                // // init_subscriber(subscriber);

                // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port.");

                // // We retrieve the port assigned to us by the OS
                // let port = listener.local_addr().unwrap().port();

                // let address = format!("http://127.0.0.1:{}", port);
                
                // let mut configuration = get_configuration().expect("Failed to read configuration."); 
                // configuration.database.database_name = Uuid::new_v4().to_string();
                
                // let connection_pool = configure_database(&configuration.database).await;

                // // Build a new email client

                // let sender_email = configuration.email_client.sender()
                //     .expect("Invalid sender email address.");

                // let timeout = configuration.email_client.timeout();

                // let email_client = EmailClient::new(
                //     configuration.email_client.base_url,
                //     sender_email, 
                //     // Pass argument from configuration 
                //     configuration.email_client.authorization_token,
                //     timeout
                // );

                // OLD CODE before wanting to reset the database while testing, to avoid unique email duplication issue
                // let connection_pool = PgPool::connect(
                //         &configuration.database.connection_string()
                //     )
                //     .await
                //     .expect("Failed to connect to Postgres.");

                // let server = run(listener, connection_pool.clone(), email_client).expect("Failed to bind address.");

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");

    // Get the port before spawning the application.
    let address = format!("http://127.0.0.1:{}", application.port());


    let _ = tokio::spawn(application.run_until_stopped());

                // TestApp {
                //     address:,
                //     db_pool: connection_pool,
                // }

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }
    // OLD VERSION BEFORE POOL
    // println!("THE PORT {:?}", port);

    // // We return the application address to the caller!
    // format!("http://127.0.0.1:{}", port)
}

async fn configure_database(config: &DatabaseSettings) -> PgPool { 
    // Create database
    let mut connection = PgConnection::connect_with(
                &config.without_db()
        )
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str()) 
        .await
        .expect("Failed to create database.");


    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db()) 
        .await
        .expect("Failed to connect to Postgres."); 
    
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    
    connection_pool
}
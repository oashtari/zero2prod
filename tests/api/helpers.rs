// use actix_web::App; 
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
// use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
// use zero2prod::email_cliendot::EmailClient;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::{get_connection_pool, Application};
use wiremock::MockServer;

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

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks { 
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url
}

// #[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded") .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Extract the confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request 
    ) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();

        assert_eq!(links.len(),1);
        let raw_link = links[0].as_str().to_owned();
        let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
        
        // Let's make sure we don't call random APIs on the web
        assert_eq!(confirmation_link.host_str().unwrap(),"127.0.0.1");
        confirmation_link.set_port(Some(self.port)).unwrap();
        confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text}
    } 
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

    let email_server = MockServer::start().await;
    // Randomize configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.HELPERS");

        // Use different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        
        // Use a random OS port
        c.application.port = 0;

        c.email_client.base_url = email_server.uri();

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

    let application_port = application.port();

    // DYNAMIC port now
    // // Get the port before spawning the application.
    // let address = format!("http://127.0.0.1:{}", application.port());


    let _ = tokio::spawn(application.run_until_stopped());

                // TestApp {
                //     address:,
                //     db_pool: connection_pool,
                // }

    TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
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
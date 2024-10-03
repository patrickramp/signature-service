// Import necessary dependencies
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use base58::ToBase58;
use ring::{digest, signature};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::{env, fs};

// Define the structure of the request payload
#[derive(Deserialize, Serialize)]
struct SignRequest {
    email: String,
}

// Define the structure of the response payload
#[derive(Serialize)]
struct SignResponse {
    signature: String,
}

// Application state containing the path to the private key file
struct AppState {
    private_key_file: String,
}

// Asynchronous handler function to sign an email address
async fn sign_email(
    req: web::Json<SignRequest>,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> impl Responder {
    // Compute the SHA-256 hash of the email
    let email_hash = digest::digest(&digest::SHA256, req.email.as_bytes());

    // Read the Ed25519 private key from the file
    let private_key = fs::read(&app_state.lock().unwrap().private_key_file)
        .expect("Failed to read private key file. Ensure it is in the correct format.");
    let key_pair = signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(&private_key)
        .expect("Failed to load private key");

    // Sign the email hash using the private key
    let signature = key_pair.sign(email_hash.as_ref());

    // Encode the signature in base58 format
    let signature_base58 = signature.as_ref().to_base58().to_uppercase();

    // Return the signature in a JSON response
    HttpResponse::Ok().json(SignResponse {
        signature: signature_base58,
    })
}

// Main function to configure and start the Actix web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Retrieve server configuration from environment variables
    let bind_to = env::var("BIND_TO").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let origin = env::var("ORIGIN").unwrap_or_else(|_| "*".to_string());

    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level));

    // Get the private key file path from command-line arguments or use default
    let args: Vec<String> = env::args().collect();
    let private_key_file = if args.len() < 2 {
        "certs/private_key.der".to_string()
    } else {
        args[1].clone()
    };

    // Check if the private key file exists
    if !Path::new(&private_key_file).exists() {
        eprintln!("Error: Private key not found at {}", private_key_file);
        std::process::exit(1);
    } else {
        println!("Private key found: {}", private_key_file);
    }

    // Split the `ORIGIN` environment variable into multiple origins
    let allowed_origins: Vec<String> = origin.split(',').map(|s| s.trim().to_string()).collect();

    // Print server startup message
    println!(
        "Signing Server starting on http://{}:{}/sign",
        bind_to, port
    );

    // Configure and start the HTTP server
    HttpServer::new(move || {
        let state = Arc::new(Mutex::new(AppState {
            private_key_file: private_key_file.clone(),
        }));

        let mut cors = Cors::default() // Configure Cross-Origin Resource Sharing (CORS)
            .allowed_methods(vec!["POST"]); // Allow only POST requests

        // Handle the wildcard case for allowing any origin
        if allowed_origins.len() == 1 && allowed_origins[0] == "*" {
            cors = cors.allow_any_origin(); // Allow any HTTP header
        } else {
            // Add each allowed origin to the CORS configuration
            for allowed_origin in &allowed_origins {
                cors = cors.allowed_origin(allowed_origin);
            }
        }

        App::new()
            .app_data(web::Data::new(state.clone())) // Share application state with handlers
            .wrap(middleware::Logger::default()) // Enable request logging
            .wrap(cors)
            .route("/sign", web::post().to(sign_email)) // Define the /sign endpoint
    })
    .bind(format!("{}:{}", bind_to, port)) // Bind server to address and port
    .expect("Error binding server") // Handle binding errors
    .run() // Start the server
    .await
}

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use base58::ToBase58;
use ring::{digest, signature};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

// Define the request payload structure
#[derive(Deserialize, Serialize)]
struct SignRequest {
    email: String,
}

// Define the response structure
#[derive(Serialize)]
struct SignResponse {
    signature: String,
}

// Define the application state containing the path to the private key file
struct AppState {
    private_key_file: String,
}

// Asynchronous handler function to sign the email address
async fn sign_email(
    req: web::Json<SignRequest>, // Incoming request JSON with email
    private_key_file: web::Data<Arc<Mutex<AppState>>>, // Application state with private key file path
) -> impl Responder {
    // Hash the provided email address using SHA-256
    let email_hash = digest::digest(&digest::SHA256, req.email.as_bytes());

    // Read and load the Ed25519 private key from the specified file
    let key_pair = signature::Ed25519KeyPair::from_pkcs8_maybe_unchecked(
        &fs::read(&private_key_file.lock().unwrap().private_key_file)
            .expect("Error reading private key file. Ensure the key is in Ed25519 DER format."),
    )
    .expect("Error loading private key");

    // Sign the email hash with the private key
    let signature = key_pair.sign(email_hash.as_ref());

    // Encode the signature in base58 format
    let signature_base58 = signature.as_ref().to_base58().to_uppercase();

    // Return the signature in JSON format
    HttpResponse::Ok().json(SignResponse {
        signature: signature_base58,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Define server parameters from environment variables
    let bind_to = env::var("BIND_TO").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8888".to_string());

    // Collect command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // Ensure that a private key file path is provided
    if args.len() < 2 {
        eprintln!("Error: Missing argument. Usage: <program> <path/to/private.key>");
        std::process::exit(1);
    }

    let private_key_file = args[1].clone();

    // Verify that the private key file exists
    if !Path::new(&private_key_file).exists() {
        eprintln!("Error: Private key not found at {}", private_key_file);
        return Ok(());
    } else {
        println!("Private key found at {}", private_key_file);
    }

    // Print server startup message
    println!("Signing Server running on http://{}:{}/sign", bind_to, port);

    // Set logging environment variable
    std::env::set_var("RUST_LOG", "actix_web=info");

    // Configure and start the HTTP server
    HttpServer::new(move || {
        let state = Arc::new(Mutex::new(AppState {
            private_key_file: private_key_file.clone(),
        }));

        App::new()
            .app_data(web::Data::new(state.clone())) // Share application state with handlers
            .wrap(
                Cors::default() // Set up CORS policy
                    .allow_any_origin() // Allow requests from any origin
                    .allow_any_method() // Allow any HTTP method
                    .allow_any_header(), // Allow any headers
            )
            .wrap(Logger::default()) // Log all incoming requests
            .route("/sign", web::post().to(sign_email)) // Define route for signing emails
    })
    .bind(format!("{}:{}", bind_to, port))? // Bind the server to the specified address
    .run() // Start the server
    .await
}

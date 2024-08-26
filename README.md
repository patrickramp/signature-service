# Signing-Service 

This is a simple signing service in Rust which can be used to sign email addresses for my other project signature-presentation. 

## Usage
Use the environment variable BIND_TO to set the address to provide the service (Defaults to 0.0.0.0), and PORT to set the service port (Defaults to 8888). 

## Generate private key
This application uses Ed25519 private key to sign email addresses. The key must be in .der format NOT .pem. To generate a .der formatted key with OpenSSL use:   

```openssl genpkey -algorithm ED25519 -outform DER -out private_key.der```

# Signing-Service 

This is a simple signing service in Rust which can be used to sign email adrresses for my other project signature-presentation. 

## Generate private key
This applicaton uses Ed25519 private key to sign email addresses. The key must be in .der format NOT .pem. To generate a .der formatted key with OpenSSL use:   

`openssl genpkey -algorithm ED25519 -outform DER -out private_key.der`.


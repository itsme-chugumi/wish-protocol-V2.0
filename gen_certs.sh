#!/bin/bash
mkdir -p ~/.wish

# 1. Generate CA
openssl req -x509 -newkey rsa:4096 -keyout ~/.wish/ca.key -out ~/.wish/ca.pem -days 365 -nodes -subj "/CN=WishCA"

# 2. Generate Server Key and CSR
openssl req -newkey rsa:4096 -keyout ~/.wish/key.pem -out ~/.wish/server.csr -nodes -subj "/CN=localhost"

# 3. Sign Server Cert with CA
# Create extfile for SAN
echo "subjectAltName=DNS:localhost" > ~/.wish/server.ext

openssl x509 -req -in ~/.wish/server.csr -CA ~/.wish/ca.pem -CAkey ~/.wish/ca.key -CAcreateserial -out ~/.wish/cert.pem -days 365 -sha256 -extfile ~/.wish/server.ext

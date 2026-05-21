$SUBJ = "/C=RU/ST=Test/L=Test/O=Selecit/OU=/CN=localhost/emailAddress="
$CERTS_DIR = "certs"

if (-Not (Test-Path -Path $CERTS_DIR)) {
    New-Item -ItemType Directory -Path $CERTS_DIR
}

# CA
openssl req -x509 -newkey rsa:4096 -days 36500 -keyout $CERTS_DIR/ca-key.pem -out $CERTS_DIR/ca-cert.pem -nodes -subj $SUBJ

# CSR
openssl req -newkey rsa:4096 -keyout $CERTS_DIR/server-key.pem -out $CERTS_DIR/server-req.pem -subj $SUBJ -nodes

# server cert
openssl x509 -req -in $CERTS_DIR/server-req.pem -CA $CERTS_DIR/ca-cert.pem -CAkey $CERTS_DIR/ca-key.pem -CAcreateserial -out $CERTS_DIR/server-cert.pem -extfile localhost.ext
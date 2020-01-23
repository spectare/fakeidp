# OIDC Token Mock service for testing

This is a HTTP(S) based service with a valid key and discovery endpoint that allows
to generate JWT tokens based on the claims you like to receive. 

## Running the service

Running the binary works follows: 

```bash
oidc-token-test-service 0.2
Allows to generate any valid JWT for OIDC

USAGE:
    oidc-token-test-service [OPTIONS] [keyfile]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bind <bind>    Sets the host or IP number to bind to [default: 0.0.0.0]
    -p, --port <port>    Sets the port to listen to [default: 8080]

ARGS:
    <keyfile>    Location of the RSA DER keypair as a file
```

### Generate keys

The mock service makes use of DER encoded key files. The easiest way to generate these are the openssl tool

```
openssl genpkey -algorithm RSA \
                -pkeyopt rsa_keygen_bits:2048 \
                -outform der \
                -out private_key.der
```


### The other option is to run it as a DOCKER container:

```bash
docker run -p9090:8080 -e BIND=0.0.0.0 -e PORT=9090 spectare/oidc-token-test-service:latest
```

where BIND and PORT are environment variables that allow you to change the endpoint binding and address within the container. 
Note that you need to expose the port you choose. 

## Example


The service runs by default on port 8080 and in order to generate a token, you post the required claimset 
to the /token endpoint

```bash
curl -d "@claim.json" -X POST http://`hostname -f`:8080/token
```
where claim.json contains the claimset:
```json
{
  "iss": "http://localhost:8080/mock",
  "sub": "CgVhZG1pbhIFbG9jYWw",
  "aud": "cafienne-ui",
  "exp": 1576568495,
  "iat": 1576482095,
  "at_hash": "zqKhL-sV6TNJUFQSF7PwLQ",
  "email": "admin@example.com",
  "email_verified": true,
  "name": "admin"
}
```

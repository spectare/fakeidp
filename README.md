# OIDC compatible Fake IdP service for testing

This is a HTTP(S) based service with a valid key and discovery endpoint that allows
to generate JWT tokens based on the claims you like to receive. 

WARNING: This is a **test** service, don't use the keys and the service as such in a 
production setup as it allows you to create any kind of JWT token signed by the given keys. 

## Running the service

Running the binary works follows: 

```bash
oidc-token-test-service
Allows to generate any valid JWT for OIDC

USAGE:
    oidc-token-test-service [OPTIONS] [keyfile]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bind <bind>    Sets the host or IP number to bind to [default: 0.0.0.0]
    -p, --port <port>    Sets the port to listen to [default: 8080]
    -h, --host <url> Full base URL of the host the service is found, like https://accounts.google.com
    -s, --static <folder> Sets the folder used to serve the static files from.

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
Note that a keypair is provided by default. 

### The other option is to run it as a DOCKER container:

```bash
docker run -p9090:9090 -e BIND=0.0.0.0 -e PORT=9090 -e EXPOSED_HOST=http://localhost:9090 spectare/oidc-token-test-service:latest
```

where BIND and PORT are environment variables that allow you to change the endpoint binding and address within the container. 
Note that you need to expose the port you choose and match that with the exposed host name/port.
EXPOSED_HOST is  the base URL used by the outside world to find the ./well-known/openid-configuration and the keys. 

## Use it for manual OIDC Login

At this moment it is possible to use a simple implicit flow (response_type=token%20id_token&scope=openid) and trigger
a login screen. Within the screen you can setup your sub(ject) - most of the times your account ID and your name. 
the .well-known/openid-configuration endpoint returns the proper authorization endpoint (/auth)

NOTE: PKCE FLOW IS NOT YET SUPPORTED

## Example for JWT token creation

The service runs by default on port 8080 and in order to generate a token, you post the required claimset 
to the /token endpoint

```bash
curl -d "@claim.json" -X POST http://`hostname -f`:9090/token
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

## Example for userinfo

When you need to mock your userinfo call, you can create a token with the above example and thereafter
do a GET on the /userinfo enpoint with an 'Authorization' header including 'Bearer <jwt>' where <jwt> is the
generated token of the example. 

Note that your claims need to contain the fields you want to return for userinfo. 
Currently supported are:

```json
{
    "iss": "http://localhost:8080",
    "sub": "F82E617D-DEAF-4EE6-8F96-CF3409060CA2",
    "email": "admin@example.com",
    "email_verified": true,
    "name": "Arie Ministrone"
}
```
Name, email and email_verified. The other 2 are required for generation of the token and are used in the validation. 


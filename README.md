#OIDC Token Mock service for testing

This is a HTTP(S) based service with a valid key and discovery endpoint that allows
to generate JWT tokens based on the claims you like to receive. 

Example


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

#Generate keys

The mock service makes use of DER encoded key files. The easiest way to generate these are the openssl tool

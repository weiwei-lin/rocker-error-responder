# rocket_simple_responder
A rust that supports deriving simple rocket responder.

## Example
```rust
use rocket_simple_responder::SimpleResponder;
use thiserror::Error;

#[derive(Debug, Error, SimpleResponder)]
#[response(code = 500)]
enum QueryEndpointError {
    #[error("bad request")]
    #[response(code = 400)]
    BadRequest,
    #[error("auth error happened")]
    AuthError(#[response(delegate)] AuthenticationError),
    #[error("something bad happened")]
    InternalServerError,
}

#[derive(Debug, Error, SimpleResponder)]
enum AuthenticationError {
    #[error("only admin can access this endpoint")]
    #[response(code = 403)]
    Forbidden,
    #[error("please login")]
    #[response(code = 401)]
    Unauthorized,
}
```

# rocket_simple_responder
A rust that supports deriving simple rocket responder.

## Example
```rust
use rocket_simple_responder::{GetStatus, SimpleResponder};
use thiserror::Error;

#[derive(Debug, Error, GetStatus, SimpleResponder)]
enum QueryEndpointError {
    #[error("only admin can access this endpoint")]
    #[simple_responder(code = 403)]
    AuthenticationError,
    #[error("something bad happened")]
    #[simple_responder(code = 500)]
    InternalServerError,
}
```

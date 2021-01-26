use std::fmt::Debug;

use rocket::{
    get,
    http::{ContentType, Status},
    local::asynchronous::Client,
    routes,
};
use rocket_simple_responder::SimpleResponder;
use thiserror::Error;

#[derive(Debug, Error, SimpleResponder)]
enum WithInternalServerError<T, E>
where
    T: Debug,
    E: WithType + Debug,
    E::A: Debug,
{
    #[error("internal server error")]
    #[response(code = 500)]
    InternalServerError(String),
    #[error("internal server error")]
    OtherError(#[response(delegate)] T),
    #[error("internal server error")]
    OtherError2(#[response(delegate)] E::A),
}

#[derive(Debug)]
struct Dummy;
impl WithType for Dummy {
    type A = String;
}

trait WithType {
    type A;
}

#[derive(Debug, Error, SimpleResponder)]
enum AuthError {
    #[error("unauthorized")]
    #[response(code = 401)]
    Unauthorized,
    #[error("forbidden")]
    #[response(code = 403)]
    Forbidden,
}

#[get("/")]
fn case1_route() -> WithInternalServerError<AuthError, Dummy> {
    WithInternalServerError::OtherError(AuthError::Forbidden)
}

#[tokio::test]
async fn case1() {
    let rocket = rocket::ignite().mount("/", routes![case1_route]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(
        response.into_string().await,
        Some(AuthError::Forbidden.to_string())
    );
}

#[get("/")]
fn case2_route() -> WithInternalServerError<AuthError, Dummy> {
    WithInternalServerError::InternalServerError("".into())
}

#[tokio::test]
async fn case2() {
    let rocket = rocket::ignite().mount("/", routes![case2_route]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(
        response.into_string().await,
        Some(
            WithInternalServerError::<AuthError, Dummy>::InternalServerError("".into()).to_string()
        )
    );
}

#[get("/")]
fn case3_route() -> WithInternalServerError<AuthError, Dummy> {
    WithInternalServerError::OtherError2("content".into())
}

#[tokio::test]
async fn case3() {
    let rocket = rocket::ignite().mount("/", routes![case3_route]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(response.into_string().await, Some("content".into()));
}

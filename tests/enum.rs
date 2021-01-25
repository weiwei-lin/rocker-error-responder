use rocket::{
    get,
    http::{ContentType, Status},
    local::asynchronous::Client,
    routes,
};
use rocket_simple_responder::SimpleResponder;
use thiserror::Error;

#[derive(Debug, Error, SimpleResponder)]
#[response(code = 500)]
enum Error {
    #[error("internal server error")]
    #[response(code = 400)]
    BadRequest(String),
    #[error("not found")]
    #[response(code = 404)]
    NotFound,
    #[error("auth error")]
    #[response(delegate = .0)]
    Auth(AuthError),
    #[error("other")]
    Other,
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
fn case1_route() -> Error {
    Error::BadRequest("".into())
}

#[tokio::test]
async fn case1() {
    let rocket = rocket::ignite().mount("/", routes![case1_route]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::BadRequest);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(
        response.into_string().await,
        Some(Error::BadRequest("".into()).to_string())
    );
}

#[get("/")]
fn case2_route() -> Error {
    Error::NotFound
}

#[tokio::test]
async fn case2() {
    let rocket = rocket::ignite().mount("/", routes![case2_route]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(
        response.into_string().await,
        Some(Error::NotFound.to_string())
    );
}

#[get("/")]
fn case3_route() -> Error {
    Error::Other
}

#[tokio::test]
async fn case3() {
    let rocket = rocket::ignite().mount("/", routes![case3_route]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(response.into_string().await, Some(Error::Other.to_string()));
}

#[get("/")]
fn case4_route() -> Error {
    Error::Auth(AuthError::Forbidden)
}

#[tokio::test]
async fn case4() {
    let rocket = rocket::ignite().mount("/", routes![case4_route]);
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

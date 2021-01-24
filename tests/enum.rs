use rocket::{
    get,
    http::{ContentType, Status},
    local::asynchronous::Client,
    routes,
};
use rocket_simple_responder::SimpleResponder;
use thiserror::Error;

#[derive(Debug, Error, SimpleResponder)]
#[error("error message")]
#[response(code = 500)]
enum Error {
    #[error("internal server error")]
    #[response(code = 400)]
    BadRequest,
    #[error("not found")]
    #[response(code = 404)]
    NotFound,
    #[error("other")]
    Other,
}

#[get("/")]
fn case1_route() -> Error {
    Error::BadRequest
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
        Some(Error::BadRequest.to_string())
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

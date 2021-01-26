use rocket::{
    get,
    http::{ContentType, Status},
    local::asynchronous::Client,
    routes,
};
use rocket_simple_responder::SimpleResponder;
use thiserror::Error;

#[derive(Debug, Error, SimpleResponder)]
enum WithInternalServerError<T>
where
    T: std::fmt::Debug,
{
    #[error("internal server error")]
    #[response(code = 500)]
    InternalServerError(String),
    #[error("internal server error")]
    #[response(delegate = .0)]
    OtherError(T),
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
fn case1_route() -> WithInternalServerError<AuthError> {
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
fn case2_route() -> WithInternalServerError<AuthError> {
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
        Some(WithInternalServerError::<AuthError>::InternalServerError("".into()).to_string())
    );
}

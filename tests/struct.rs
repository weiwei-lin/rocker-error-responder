use rocket::{
    get,
    http::{ContentType, Status},
    local::asynchronous::Client,
    routes,
};
use rocket_simple_responder::SimpleResponder;
use thiserror::Error;

#[tokio::test]
async fn case1() {
    #[derive(Debug, Error, SimpleResponder)]
    #[error("error message")]
    #[response(code = 500)]
    struct Error;

    #[get("/")]
    fn test_response() -> Error {
        Error
    }
    let rocket = rocket::ignite().mount("/", routes![test_response]);
    let client = Client::untracked(rocket)
        .await
        .expect("valid rocket instance");
    let response = client.get("/").dispatch().await;
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::Plain));

    assert_eq!(response.into_string().await, Some(Error.to_string()));
}

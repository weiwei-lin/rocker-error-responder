use rocket::http::Status;
pub use rocket_simple_responder_derive::{GetStatus, SimpleResponder};

pub trait GetStatus {
    fn get_status(&self) -> Status;
}

use crate::{request::Request, response::Response};

pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Option<Response>;
}

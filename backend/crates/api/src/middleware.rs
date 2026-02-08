use axum::http::Request;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower_http::request_id::{MakeRequestId, RequestId};

#[derive(Clone, Default)]
pub struct RequestIdGenerator {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let _id = self.counter.fetch_add(1, Ordering::SeqCst);
        let ulid = ulid::Ulid::new();
        Some(RequestId::new(ulid.to_string().parse().unwrap()))
    }
}

pub mod stream;
pub mod thread;

use salvo::prelude::*;

pub fn authed_root(path: impl Into<String>) -> Router {
    Router::with_path(path)
        .push(
            Router::with_path("streams")
                .get(stream::list)
                .post(stream::create)
                .push(Router::with_path("buckets").post(stream::upload))
                .push(Router::with_path(r"buckets/<*path>").get(stream::serve_file))
                .push(
                    Router::with_path(r"<id:/\d+/>")
                        .get(stream::show)
                        .patch(stream::update)
                        .put(stream::update),
                ),
        )
        .push(
            Router::with_path("threads")
                .get(thread::list)
                .post(thread::create)
                .push(
                    Router::with_path(r"<id:/\d+/>")
                        .get(thread::show)
                        .patch(thread::update)
                        .put(thread::update),
                ),
        )
}

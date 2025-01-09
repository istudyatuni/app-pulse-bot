use std::future::Future;

use tokio_util::sync::CancellationToken;

pub async fn spawn_with_token<R>(token: CancellationToken, f: impl Future<Output = R>) {
    tokio::select! {
        _ = token.cancelled() => {},
        _ = f => {},
    }
}

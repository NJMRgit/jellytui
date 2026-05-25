# jellyfin-client

Async Rust client for the [Jellyfin](https://jellyfin.org) media server HTTP API.

This crate covers the endpoints needed to build a media browser or player:
authentication, library browsing (libraries, resume, next-up, latest), item
lookup, search, direct stream URLs, external subtitle URLs, and the
session-reporting endpoints (`Playing` / `Progress` / `Stopped` / `PlayedItems`).

It is intentionally small — not a full code-generated binding of every
Jellyfin endpoint. If you need broader coverage, treat this as a starting
point.

## Usage

```rust,no_run
use jellyfin_client::{ClientInfo, JellyfinClient};

#[tokio::main]
async fn main() -> Result<(), jellyfin_client::Error> {
    let info = ClientInfo::new("my-app", env!("CARGO_PKG_VERSION"))
        .with_device_id("host-abc123");

    let mut client = JellyfinClient::new("https://jellyfin.example.com", info);
    client.authenticate("alice", "hunter2").await?;

    for view in client.get_user_views().await? {
        println!("{}: {}", view.r#type, view.name);
    }
    Ok(())
}
```

After a successful `authenticate`, persist `client.access_token()` and
`client.user_id()` so subsequent runs can restore the session via
`JellyfinClient::with_token`.

## TLS

By default the crate enables `reqwest`'s native TLS backend. To use rustls
instead:

```toml
jellyfin-client = { version = "0.1", default-features = false, features = ["rustls-tls"] }
```

## Error handling

All fallible operations return `Result<T, jellyfin_client::Error>`. Match on
the variant — in particular, `Error::Unauthorized` indicates the stored token
is no longer valid and the user must re-authenticate.

## License

MIT OR Apache-2.0.

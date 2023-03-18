use async_once::AsyncOnce;
use lazy_static::lazy_static;
use redis::aio::MultiplexedConnection;

lazy_static! {
    pub static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
        Client {
            conn: redis::Client::open("redis://127.0.0.1:6379/")
                .unwrap()
                .get_multiplexed_tokio_connection()
                .await
                .expect("Expected redis client to be initialized"),
        }
    });
}

pub struct Client {
    pub(crate) conn: MultiplexedConnection,
}

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use redis::aio::MultiplexedConnection;
use serde::Deserialize;
use shared_kernel::configuration::config;

#[derive(Deserialize)]
struct Settings {
    redis: RedisConfig,
}

#[derive(Deserialize)]
struct RedisConfig {
    host: String,
}
lazy_static! {
    static ref SETTINGS: RedisConfig = config::<Settings>().expect("settings to be defined").redis;
    pub static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
        Client {
            conn: redis::Client::open(SETTINGS.host.as_ref())
                .unwrap()
                .get_multiplexed_tokio_connection()
                .await
                .expect("Expected redis client to be initialized"),
        }
    });
}

#[derive(Clone)]
pub struct Client {
    pub(crate) conn: MultiplexedConnection,
}

impl Client {
    pub fn connection(&self) -> MultiplexedConnection {
        self.conn.clone()
    }
}

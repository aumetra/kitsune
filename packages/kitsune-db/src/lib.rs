//!
//! Definition of data repositories as dyn-compatible traits
//!

use async_trait::async_trait;
use diesel_async::pooled_connection::bb8;
use frunk::LabelledGeneric;
use quick_error::quick_error;
use time::OffsetDateTime;
use uuid::Uuid;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        /// Pool errored out
        Pool(err: bb8::RunError) {
            from()
        }

        /// Query error
        Query(err: diesel::result::Error) {
            from()
        }

        /// Something failed
        Other(err: Box<dyn std::error::Error + Send + Sync>)
    }
}

#[derive(LabelledGeneric)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub username: &'a str,
    pub email: &'a str,
    pub hashed_password: &'a str,
}

#[derive(Clone, Debug, LabelledGeneric, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: NewUser<'_>) -> Result<User, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

static_assertions::assert_obj_safe!(UserRepository);

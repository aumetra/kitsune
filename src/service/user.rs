use argon2::{
    Argon2,
    password_hash::{self, PasswordHasher, SaltString},
};
use bon::Builder;
use garde::Validate;
use kitsune_db::UserRepository;
use quick_error::quick_error;
use rand::rngs::OsRng;
use std::sync::Arc;
use uuid::Uuid;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        /// Password hashing failed
        PasswordHashing(err: password_hash::Error) {
            from()
        }

        /// Repository issue
        Repository(err: kitsune_db::Error) {
            from()
        }

        /// Validation failed
        Validation(err: garde::Report) {
            from()
        }
    }
}

#[derive(Builder, Clone)]
pub struct Service {
    allow_non_ascii: bool,
    #[builder(into)]
    repository: Arc<dyn UserRepository>,
}

pub struct NewUserContext {
    allow_non_ascii: bool,
}

#[inline]
fn conditional_ascii_check(value: &str, ctx: &NewUserContext) -> garde::Result {
    if ctx.allow_non_ascii {
        return Ok(());
    }

    garde::rules::ascii::apply(&value, ())
}

#[derive(Builder, Validate)]
#[garde(context(NewUserContext))]
pub struct NewUser {
    #[garde(
        custom(conditional_ascii_check),
        length(min = 1, max = 64),
        pattern(r"^[\p{L}\p{N}\.]+$")
    )]
    username: String,
    #[garde(email)]
    email: String,
    #[garde(length(min = 1))]
    password: String,
}

impl Service {
    pub async fn register(&self, new_user: NewUser) -> Result<(), Error> {
        new_user.validate_with(&NewUserContext {
            allow_non_ascii: self.allow_non_ascii,
        })?;

        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Argon2::default()
            .hash_password(new_user.password.as_bytes(), &salt)?
            .to_string();

        self.repository
            .create(kitsune_db::NewUser {
                id: Uuid::now_v7(),
                username: &new_user.username,
                email: &new_user.email,
                hashed_password: &hashed_password,
            })
            .await?;

        Ok(())
    }
}

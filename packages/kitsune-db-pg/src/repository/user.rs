use crate::{PgPool, schema::users};
use async_trait::async_trait;
use bon::Builder;
use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl, Selectable, SelectableHelper,
    prelude::{Insertable, Queryable},
};
use diesel_async::RunQueryDsl;
use frunk::{LabelledGeneric, labelled::Transmogrifier};
use kitsune_db::UserRepository;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Insertable, LabelledGeneric)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    id: Uuid,
    username: &'a str,
    email: &'a str,
    hashed_password: &'a str,
}

#[derive(LabelledGeneric, Selectable, Queryable)]
struct User {
    id: Uuid,
    username: String,
    email: String,
    hashed_password: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Builder)]
pub struct Repository {
    pool: PgPool,
}

#[async_trait]
impl UserRepository for Repository {
    async fn create(
        &self,
        user: kitsune_db::NewUser<'_>,
    ) -> Result<kitsune_db::User, kitsune_db::Error> {
        let insertable: NewUser = user.transmogrify();

        let mut conn = self.pool.get().await?;
        let user = diesel::insert_into(users::table)
            .values(insertable)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await?;

        Ok(user.transmogrify())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<kitsune_db::User>, kitsune_db::Error> {
        let mut conn = self.pool.get().await?;
        let maybe_user = users::table
            .find(id)
            .select(User::as_select())
            .get_result(&mut conn)
            .await
            .optional()?;

        Ok(maybe_user.map(Transmogrifier::transmogrify))
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<kitsune_db::User>, kitsune_db::Error> {
        let mut conn = self.pool.get().await?;
        let maybe_user = users::table
            .filter(users::username.eq(username))
            .select(User::as_select())
            .get_result(&mut conn)
            .await
            .optional()?;

        Ok(maybe_user.map(Transmogrifier::transmogrify))
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), kitsune_db::Error> {
        let mut conn = self.pool.get().await?;
        diesel::delete(users::table.find(id))
            .execute(&mut conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use super::Repository;
    use kitsune_db::{NewUser, UserRepository};
    use uuid::Uuid;

    async fn connect() -> impl UserRepository {
        let pg_url = env::var("PG_DB_URL").unwrap();
        let pool = crate::connect(&pg_url).await.unwrap();

        Repository::builder().pool(pool).build()
    }

    #[tokio::test]
    async fn create() {
        let repo = connect().await;

        let id = Uuid::now_v7();
        let created = repo
            .create(NewUser {
                id,
                username: "test",
                email: "test@test.com",
                hashed_password: "abc",
            })
            .await
            .unwrap();

        let retrieved_user = repo.find_by_id(id).await.unwrap();

        assert_eq!(Some(created), retrieved_user);
    }

    #[tokio::test]
    async fn find_by_username() {
        let repo = connect().await;

        let id = Uuid::now_v7();
        let created = repo
            .create(NewUser {
                id,
                username: "test",
                email: "test@test.com",
                hashed_password: "abc",
            })
            .await
            .unwrap();

        let retrieved_user = repo.find_by_username("test").await.unwrap();
        assert_eq!(Some(created), retrieved_user);

        let retrieved_user = repo.find_by_username("meow").await.unwrap();
        assert!(retrieved_user.is_none());
    }

    #[tokio::test]
    async fn username_collation_works() {
        let repo = connect().await;

        let result = repo
            .create(NewUser {
                id: Uuid::now_v7(),
                username: "test",
                email: "test@test.com",
                hashed_password: "abc",
            })
            .await;

        assert!(result.is_ok());

        let result = repo
            .create(NewUser {
                id: Uuid::now_v7(),
                username: "tEst",
                email: "test123@test.com",
                hashed_password: "abc",
            })
            .await;

        assert!(result.is_err());

        let result = repo
            .create(NewUser {
                id: Uuid::now_v7(),
                username: "TèSt",
                email: "test123@test.com",
                hashed_password: "abc",
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete() {
        let repo = connect().await;

        let id = Uuid::now_v7();
        repo.create(NewUser {
            id,
            username: "test",
            email: "test@test.com",
            hashed_password: "abc",
        })
        .await
        .unwrap();

        let retrieved_user = repo.find_by_id(id).await.unwrap();
        assert!(retrieved_user.is_some());

        repo.delete_by_id(id).await.unwrap();

        let retrieved_user = repo.find_by_id(id).await.unwrap();
        assert!(retrieved_user.is_none());
    }
}

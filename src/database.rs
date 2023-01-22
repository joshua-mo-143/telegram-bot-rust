use sqlx::PgPool;
use teloxide::types::ChatId;

pub async fn create_watch(status: String, url: String, user_id: ChatId, connection: PgPool) -> Result<(), sqlx::Error> {
    sqlx::query
          ("INSERT INTO links (url, status, user_id) VALUES ($1, $2, $3)")
          .bind(url)
          .bind(status)
          .bind(user_id.to_string())
          .execute(&connection)
          .await?;

Ok(())
}

pub async fn delete_watch(url: String, user_id: ChatId, connection: PgPool) -> Result<(), sqlx::Error> {

sqlx::query("DELETE FROM links WHERE url = $1 AND user_id = $2")
      .bind(url)
      .bind(user_id.to_string())
      .execute(&connection)
      .await?;

Ok(())
}

pub async fn get_all_watch(user_id: ChatId, connection: PgPool) -> Result<(), sqlx::Error> {
sqlx::query("SELECT * FROM links WHERE user_id = $1")
  .bind(user_id.to_string())
  .execute(&connection)
  .await?;

  Ok(())
}
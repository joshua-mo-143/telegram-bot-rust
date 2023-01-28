use sqlx::{postgres::PgRow, PgPool, Row};
use teloxide::types::ChatId;

pub async fn create_record(
    status: String,
    url: String,
    user_id: ChatId,
    connection: PgPool,
) -> Result<(), sqlx::Error> {
    let url = if url[0..4] == *"http" {
        url
    } else {
        format!("http://{url}")
    };

    sqlx::query("INSERT INTO links (url, status, user_id) VALUES ($1, $2, $3)")
        .bind(url)
        .bind(status)
        .bind(user_id.to_string())
        .execute(&connection)
        .await?;

    Ok(())
}

pub async fn delete_record(
    url: String,
    user_id: ChatId,
    connection: PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM links WHERE url = $1 AND user_id = $2")
        .bind(url)
        .bind(user_id.to_string())
        .execute(&connection)
        .await?;

    Ok(())
}

pub async fn get_all_records(user_id: ChatId, connection: PgPool) -> Result<Vec<PgRow>, sqlx::Error> {
    let query = sqlx::query("SELECT * FROM links WHERE user_id = $1")
        .bind(user_id.to_string())
        .fetch_all(&connection);

    match query.await {
        Ok(result) => Ok(result),
        Err(err) => Err(err),
    }
}

pub fn sort_data(data: Vec<PgRow>) -> Vec<Link> {
    let mut records: Vec<Link> = Vec::new();

    for row in data.iter() {
        let record = Link {
            id: row.get("id"),
            url: row.get("url"),
            status: row.get("status"),
        };

        records.push(record);
    }
    records
}

pub struct Link {   
    pub id: i32,
    pub url: String,
    pub status: String,
}

use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::models::Post;

pub async fn create_pool(db_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
}

pub async fn add_post(pool: &PgPool, sentence: &str, show: bool) -> Result<i64, sqlx::Error> {
    let result: i64 = sqlx::query_scalar(
        "INSERT INTO posts (sentence, show_in_list) VALUES ($1, $2) RETURNING id",
    )
    .bind(sentence)
    .bind(show)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

pub async fn get_all_posts(pool: &PgPool) -> Vec<Post> {
    sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE show_in_list = TRUE ORDER BY timestamp DESC",
    )
    .fetch_all(pool)
    .await
    .expect("Failed to fetch posts.")
}

pub async fn get_posts_by_page(pool: &PgPool, page: i64, posts_per_page: i64) -> Vec<Post> {
    sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE show_in_list = TRUE ORDER BY timestamp DESC LIMIT $1 OFFSET $2",
    )
    .bind(posts_per_page)
    .bind((page - 1) * posts_per_page)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch posts.")
}

pub async fn get_posts_by_date(pool: &PgPool, date: &str, tz: &str) -> Vec<Post> {
    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d");
    if let Err(e) = parsed_date {
        eprintln!("Failed to parse date: {}", e);
        return Vec::new();
    }
    let parsed_date = parsed_date.unwrap();
    sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE DATE(timestamp AT TIME ZONE $2) = $1 AND show_in_list = TRUE ORDER BY timestamp DESC",
    )
    .bind(parsed_date)
    .bind(tz)
    .fetch_all(pool)
    .await
    .expect("Failed to fetch posts.")
}

pub async fn get_post_by_id(pool: &PgPool, id: i64) -> Option<Post> {
    sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .expect("Failed to fetch post.")
}

pub async fn get_post_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(pool)
        .await
        .expect("Failed to fetch post count.")
}

pub async fn search_posts(pool: &PgPool, query: &str) -> Result<Vec<Post>, sqlx::Error> {
    sqlx::query_as::<_, Post>(
        "SELECT id, sentence, timestamp
        FROM posts
        WHERE show_in_list = TRUE AND search_vector @@ websearch_to_tsquery('chinese', $1)
        ORDER BY ts_rank_cd(search_vector, websearch_to_tsquery('chinese', $1)) DESC",
    )
    .bind(query)
    .fetch_all(pool)
    .await
}

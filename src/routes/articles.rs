use crate::auth::Auth;
use crate::db;
use crate::db::articles::{FeedArticles, FindArticles};
use crate::errors::{Errors, FieldValidator};
use rocket::form::Form;
use rocket::serde::json::{serde_json::json, Json, Value};
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize)]
pub struct NewArticle {
    article: NewArticleData,
}

#[derive(Deserialize, Validate)]
pub struct NewArticleData {
    #[validate(length(min = 1))]
    title: Option<String>,
    #[validate(length(min = 1))]
    description: Option<String>,
    #[validate(length(min = 1))]
    body: Option<String>,
    #[serde(rename = "tagList")]
    tag_list: Vec<String>,
}

#[post("/articles", format = "json", data = "<new_article>")]
pub async fn post_articles(
    auth: Auth,
    new_article: Json<NewArticle>,
    conn: db::Conn,
) -> Result<Value, Errors> {
    let new_article = new_article.into_inner().article;

    let mut extractor = FieldValidator::validate(&new_article);
    let title = extractor.extract("title", new_article.title);
    let description = extractor.extract("description", new_article.description);
    let body = extractor.extract("body", new_article.body);
    extractor.check()?;

    let article = conn
        .run(move |c| {
            db::articles::create(
                c,
                auth.id,
                &title,
                &description,
                &body,
                &new_article.tag_list,
            )
        })
        .await;

    Ok(json!({ "article": article }))
}

/// return multiple articles, ordered by most recent first
#[get("/articles?<params..>")]
pub async fn get_articles(params: Form<FindArticles>, auth: Option<Auth>, conn: db::Conn) -> Value {
    let user_id = auth.map(|x| x.id);

    let articles = conn
        .run(move |c| db::articles::find(c, &params, user_id))
        .await;
    json!({ "articles": articles.0, "articlesCount": articles.1 })
}

#[get("/articles/<slug>")]
pub async fn get_article(slug: String, auth: Option<Auth>, conn: db::Conn) -> Option<Value> {
    let user_id = auth.map(|x| x.id);
    conn.run(move |c| {
        db::articles::find_one(c, &slug, user_id).map(|article| json!({ "article": article }))
    })
    .await
}

#[delete("/articles/<slug>")]
pub async fn delete_article(slug: String, auth: Auth, conn: db::Conn) {
    conn.run(move |c| db::articles::delete(c, &slug, auth.id))
        .await;
}

#[post("/articles/<slug>/favorite")]
pub async fn favorite_article(slug: String, auth: Auth, conn: db::Conn) -> Option<Value> {
    conn.run(move |c| {
        db::articles::favorite(c, &slug, auth.id).map(|article| json!({ "article": article }))
    })
    .await
}

#[delete("/articles/<slug>/favorite")]
pub async fn unfavorite_article(slug: String, auth: Auth, conn: db::Conn) -> Option<Value> {
    conn.run(move |c| {
        db::articles::unfavorite(c, &slug, auth.id).map(|article| json!({ "article": article }))
    })
    .await
}

#[derive(Deserialize)]
pub struct UpdateArticle {
    article: db::articles::UpdateArticleData,
}

#[put("/articles/<slug>", format = "json", data = "<article>")]
pub async fn put_articles(
    slug: String,
    article: Json<UpdateArticle>,
    auth: Auth,
    conn: db::Conn,
) -> Option<Value> {
    // TODO: check auth
    conn.run(move |c| {
        db::articles::update(c, &slug, auth.id, article.into_inner().article)
            .map(|article| json!({ "article": article }))
    })
    .await
}

#[derive(Deserialize)]
pub struct NewComment {
    comment: NewCommentData,
}

#[derive(Deserialize, Validate)]
pub struct NewCommentData {
    #[validate(length(min = 1))]
    body: Option<String>,
}

#[post("/articles/<slug>/comments", format = "json", data = "<new_comment>")]
pub async fn post_comment(
    slug: String,
    new_comment: Json<NewComment>,
    auth: Auth,
    conn: db::Conn,
) -> Result<Value, Errors> {
    let new_comment = new_comment.into_inner().comment;

    let mut extractor = FieldValidator::validate(&new_comment);
    let body = extractor.extract("body", new_comment.body);
    extractor.check()?;

    let comment = conn
        .run(move |c| db::comments::create(c, auth.id, &slug, &body))
        .await;
    Ok(json!({ "comment": comment }))
}

#[delete("/articles/<slug>/comments/<id>")]
pub async fn delete_comment(slug: String, id: i32, auth: Auth, conn: db::Conn) {
    conn.run(move |c| db::comments::delete(c, auth.id, &slug, id))
        .await;
}

#[get("/articles/<slug>/comments")]
pub async fn get_comments(slug: String, conn: db::Conn) -> Value {
    let comments = conn
        .run(move |c| db::comments::find_by_slug(c, &slug))
        .await;
    json!({ "comments": comments })
}

#[get("/articles/feed?<params..>")]
pub async fn get_articles_feed<'v>(
    params: Form<FeedArticles>,
    auth: Auth,
    conn: db::Conn,
) -> Value {
    let articles = conn
        .run(move |c| db::articles::feed(c, &params, auth.id))
        .await;
    let articles_count = articles.len();
    json!({ "articles": articles, "articlesCount": articles_count })
}

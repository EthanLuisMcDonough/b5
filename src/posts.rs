use crate::format::*;
use crate::{
    config::CONFIG,
    entities::{self, prelude::*},
    AppState,
};
use actix_web::{error::ErrorInternalServerError, get, web, HttpResponse, Result as ActixResult};
use entities::{blog_posts, users};
use sailfish::TemplateOnce;
use sea_orm::{entity::*, prelude::*, query::*, sea_query::IntoCondition, FromQueryResult};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct CursorQuery {
    before: Option<u32>,
    after: Option<u32>,
}

#[derive(FromQueryResult, Debug)]
struct PostData {
    pub post_id: u32,
    pub title: String,
    pub body: String,
    pub username: String,
    pub post_date: DateTime,
    pub description: Option<String>,
}

#[derive(FromQueryResult, Debug)]
struct CountData {
    count: i64,
}

struct PostPreview {
    data: PostData,
    read_more: bool,
}

#[derive(TemplateOnce)]
#[template(path = "posts.stpl")]
struct PostsTemplate {
    entries: Vec<PostPreview>,
    next: Option<u32>,
    prev: Option<u32>,
}

pub async fn posts_page(
    //_req: HttpRequest,
    data: web::Data<AppState>,
    query_str: web::Query<CursorQuery>,
) -> ActixResult<HttpResponse> {
    let mut query = BlogPosts::find()
        .column(users::Column::Username)
        .join(JoinType::InnerJoin, blog_posts::Relation::Users.def())
        .limit(CONFIG.page_size);

    let mut reverse = false;
    let mut entries = if let Some(id) = query_str.after {
        reverse = true;
        query
            .order_by_asc(blog_posts::Column::PostId)
            .filter(blog_posts::Column::PostId.gt(id))
    } else {
        query = query.order_by_desc(blog_posts::Column::PostId);
        if let Some(id) = query_str.before {
            query = query.filter(blog_posts::Column::PostId.lt(id));
        }
        query
    }
    .into_model::<PostData>()
    .all(&data.db)
    .await
    .map_err(ErrorInternalServerError)?;

    if reverse {
        entries = entries.into_iter().rev().collect();
    }

    let entries = entries
        .into_iter()
        .map(|mut entry| {
            let mut body = String::new();
            let read_more = render_preview(&entry.body, &mut body);
            entry.body = body;

            PostPreview {
                data: entry,
                read_more,
            }
        })
        .collect::<Vec<_>>();

    let mut next = None;
    let mut prev = None;

    // Check if any previous posts exist
    if let Some(post) = entries.last() {
        let cond = blog_posts::Column::PostId.lt(post.data.post_id);
        let has_prev = any_posts_where(&data.db, cond).await?;
        prev = Some(post.data.post_id).filter(|_| has_prev);
    }

    // Check if any later posts exist
    if let Some(post) = entries.first() {
        let cond = blog_posts::Column::PostId.gt(post.data.post_id);
        let has_next = any_posts_where(&data.db, cond).await?;
        next = Some(post.data.post_id).filter(|_| has_next);
    }

    Ok(HttpResponse::Ok().content_type("text/html").body(
        PostsTemplate {
            entries,
            next,
            prev,
        }
        .render_once()
        .unwrap(),
    ))
}

async fn any_posts_where(db: &DatabaseConnection, filter: impl IntoCondition) -> ActixResult<bool> {
    BlogPosts::find()
        .select_only()
        .column_as(blog_posts::Column::PostId.count(), "count")
        .filter(filter)
        .into_model::<CountData>()
        .one(db)
        .await
        .map(|opt| opt.filter(|data| data.count > 0).is_some())
        .map_err(ErrorInternalServerError)
}

#[derive(TemplateOnce)]
#[template(path = "post.stpl")]
struct PostTemplate {
    post: PostData,
}

#[get("/post/{id}")]
pub async fn post_page(data: web::Data<AppState>, id: web::Path<u32>) -> ActixResult<HttpResponse> {
    let post_op = BlogPosts::find()
        .column(users::Column::Username)
        .join(JoinType::InnerJoin, blog_posts::Relation::Users.def())
        .filter(blog_posts::Column::PostId.eq(*id))
        .into_model::<PostData>()
        .one(&data.db)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(if let Some(mut post) = post_op {
        post.body = render_full(&post.body);

        HttpResponse::Ok()
            .content_type("text/html")
            .body(PostTemplate { post }.render_once().unwrap())
    } else {
        HttpResponse::NotFound()
            .content_type("text/html")
            .body("not found")
    })
}

#[derive(TemplateOnce)]
#[template(path = "rss.stpl")]
struct RssTemplate {
    posts: Vec<PostData>,
}

#[get("/feed.rss")]
async fn rss(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let posts = BlogPosts::find()
        .order_by_desc(blog_posts::Column::PostId)
        .limit(CONFIG.rss_size)
        .column(users::Column::Username)
        .join(JoinType::InnerJoin, blog_posts::Relation::Users.def())
        .into_model::<PostData>()
        .all(&data.db)
        .await
        .map_err(ErrorInternalServerError)?
        .into_iter()
        .map(|mut post| {
            post.body = render_full(&post.body);
            post
        })
        .collect();

    Ok(HttpResponse::Ok()
        .content_type("application/rss+xml")
        .body(RssTemplate { posts }.render_once().unwrap()))
}

#[get("/")]
async fn home(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    posts_page(data, web::Query(CursorQuery::default())).await
}

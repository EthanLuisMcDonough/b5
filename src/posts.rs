use crate::{
    entities::{self, prelude::*},
    AppState,
};
use actix_web::{
    error::{self, ErrorInternalServerError},
    get, web, App, HttpRequest, HttpResponse, Result as ActixResult,
};
use entities::{blog_posts, users};
use pulldown_cmark::{
    html::push_html, CowStr, Event as MarkEvent, InlineStr, Options as MarkOption, Parser,
};
use sailfish::TemplateOnce;
use sea_orm::{entity::*, prelude::*, query::*, sea_query::IntoCondition, FromQueryResult};
use serde::Deserialize;

const PAGE_SIZE: u64 = 5;
const PREVIEW_SIZE: usize = 400;

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

fn take_words(left: &mut usize, cow: &str) -> usize {
    if *left == 0 {
        return 0;
    }

    let mut in_word = false;
    let mut decr = || {
        *left -= 1;
        *left == 0
    };

    for (ind, c) in cow.char_indices() {
        if c.is_whitespace() {
            if in_word {
                if decr() {
                    return ind;
                }
                in_word = false;
            }
        } else {
            in_word = true;
        }
    }

    if in_word {
        decr();
    }

    cow.len()
}

/// Truncates markdown text/html/code token
/// See https://docs.rs/pulldown-cmark/latest/pulldown_cmark/enum.CowStr.html
fn trunc_cow(cow: CowStr<'_>, len: usize, ellipses: bool) -> CowStr<'_> {
    if cow.len() == len {
        return cow;
    } else if len == 0 {
        return CowStr::Borrowed(if ellipses { "..." } else { "" });
    }

    match cow {
        CowStr::Borrowed(b) if !ellipses => CowStr::Borrowed(&b[0..len]),
        CowStr::Inlined(inl) if !ellipses => CowStr::Inlined(
            InlineStr::try_from(&inl[0..len]).expect("slice should not exceed inline boundries"),
        ),
        _ => {
            let slice = &cow[0..len];
            InlineStr::try_from(slice)
                .map(CowStr::Inlined)
                .ok()
                .unwrap_or_else(|| {
                    let mut s = String::from(slice);
                    if ellipses {
                        s.push_str("...");
                    }
                    CowStr::Boxed(s.into_boxed_str())
                })
        }
    }
}

fn post_options() -> MarkOption {
    MarkOption::ENABLE_STRIKETHROUGH
}

pub async fn posts_page(
    //_req: HttpRequest,
    data: web::Data<AppState>,
    query_str: web::Query<CursorQuery>,
) -> ActixResult<HttpResponse> {
    let mut query = BlogPosts::find()
        .column(users::Column::Username)
        .join(JoinType::InnerJoin, blog_posts::Relation::Users.def())
        .limit(PAGE_SIZE);

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
            let mut words = PREVIEW_SIZE;
            let mut level = 0;
            let mut shortened = false;

            let truncated = Parser::new_ext(&entry.body, post_options()).map_while(|event| {
                if level == 0 && words == 0 {
                    shortened = true;
                    None
                } else {
                    let ellipses = words > 0;
                    Some(match event {
                        MarkEvent::Code(text) => {
                            let len = take_words(&mut words, &text);
                            MarkEvent::Code(trunc_cow(text.clone(), len, ellipses))
                        }
                        MarkEvent::Text(text) | MarkEvent::Html(text) => {
                            let len = take_words(&mut words, &text);
                            MarkEvent::Text(trunc_cow(text.clone(), len, ellipses))
                        }
                        // Do not show any content after a line break
                        MarkEvent::Rule => {
                            shortened = true;
                            return None;
                        }
                        MarkEvent::Start(_) => {
                            level += 1;
                            event
                        }
                        MarkEvent::End(_) => {
                            level -= 1;
                            event
                        }
                        _ => event,
                    })
                }
            });

            let mut body = String::new();
            push_html(&mut body, truncated);

            entry.body = body;

            PostPreview {
                data: entry,
                read_more: shortened,
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
        let events = Parser::new_ext(&post.body, post_options()).map(|event| match event {
            MarkEvent::Html(s) => MarkEvent::Text(s),
            _ => event,
        });

        let mut body = String::new();
        push_html(&mut body, events);

        post.body = body;

        HttpResponse::Ok()
            .content_type("text/html")
            .body(PostTemplate { post }.render_once().unwrap())
    } else {
        HttpResponse::NotFound()
            .content_type("text/html")
            .body("not found")
    })
}

#[get("/")]
async fn home(data: web::Data<AppState>) -> ActixResult<HttpResponse> {
    posts_page(data, web::Query(CursorQuery::default())).await
}

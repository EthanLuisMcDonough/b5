use pulldown_cmark::{
    html::push_html, CowStr, Event as MarkEvent, InlineStr, Options as MarkOption, Parser,
};

const PAGE_SIZE: u64 = 5;
const PREVIEW_SIZE: usize = 400;
const RSS_SIZE: u64 = 25;

/// Takes a number of words as long as there are words "left"
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

pub fn render_preview(full_body: &str, buffer: &mut String) -> bool {
    let mut words = PREVIEW_SIZE;
    let mut level = 0;
    let mut shortened = false;

    let truncated = Parser::new_ext(full_body, post_options()).map_while(|event| {
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

    push_html(buffer, truncated);
    shortened
}

pub fn render_full(body: &str) -> String {
    let events = Parser::new_ext(body, post_options()).map(|event| match event {
        MarkEvent::Html(s) => MarkEvent::Text(s),
        _ => event,
    });

    let mut body = String::new();
    push_html(&mut body, events);
    body
}

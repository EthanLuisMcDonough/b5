pub fn tagline() -> &'static str {
    use rand::seq::IteratorRandom;
    let mut rng = rand::thread_rng();
    include_str!("../config/taglines.txt")
        .split('\n')
        .filter(|e| e.len() > 0)
        .choose(&mut rng)
        .unwrap_or("No taglines found")
}

// Sailfish doesn't play nice with include_str inside templates
// so these manual exports are needed
pub const PAGE_TITLE: &'static str = include_str!("../config/site_title.txt");
pub const SITE_URL: &'static str = include_str!("../config/site_url.txt");
pub const WEBMASTER_EMAIL: &'static str = include_str!("../config/webmaster_email.txt");

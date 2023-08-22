pub async fn search(song: &str) -> Option<String> {
    if song.starts_with("http") {
        return Some(song.into());
    }
    Some(format!("ytsearch:{}", song))
}
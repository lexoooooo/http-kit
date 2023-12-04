pub const fn guess(extension: &[u8]) -> Option<&'static str> {
    match extension {
        b"aac" => Some("audio/aac"),
        b"avi" => Some("video/x-msvideo"),
        b"css" => Some("text/css"),
        b"gif" => Some("image/gif"),
        b"jpeg" | b"jpg" => Some("image/jpeg"),
        b"js" => Some("text/javascript"),
        b"json" => Some("application/json"),
        b"mp3" => Some("audio/mpeg"),
        b"mp4" => Some("video/mp4"),
        b"png" => Some("image/png"),
        b"svg" => Some("image/svg+xml"),
        b"ttf" => Some("font/ttf"),
        b"txt" => Some("text/plain"),
        b"wav" => Some("audio/wav"),
        b"webp" => Some("image/webp"),
        _ => None,
    }
}

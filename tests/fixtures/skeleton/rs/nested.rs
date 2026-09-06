pub fn outer(input: i64) -> i64 {
    let factor = 2i64;
    fn inner(x: i64) -> i64 {
        x * factor
    }
    let closure = |y: i64| {
        let z = inner(y);
        z + 1
    };
    closure(input)
}

pub async fn fetch(url: &str) -> String {
    let body = http_get(url).await;
    body
}

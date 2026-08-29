use rand::distr::Alphanumeric;
use rand::RngExt;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

async fn send_email(email: String) -> anyhow::Result<String> {
    for _ in 0..3{
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"));
        headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"));
        headers.insert("origin", HeaderValue::from_static("https://www.raccoongame.com"));
        headers.insert("priority", HeaderValue::from_static("u=1, i"));
        headers.insert("referer", HeaderValue::from_static("https://www.raccoongame.com/login?redirect_uri=https%3A%2F%2Fwww.raccoongame.com%2Fweb2%2Fdist%2F%23%2Fplatform%2Fcloudgame"));
        headers.insert("sec-ch-ua", HeaderValue::from_static("\"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"150\", \"Google Chrome\";v=\"150\""));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Linux\""));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36"));
        headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));

        headers.insert("cookie", HeaderValue::from_static(
            "JY-HASH=2e58d4e031b6fbfb34069e7c7f981080; \
             JY-LANG=zh; \
             raccoongame_session=eyJpdiI6IldCeENkNHN2QzJtR3NvMkxJcHYzSFE9PSIsInZhbHVlIjoiTlZpR0tnNWRJSlA4d1doXC9PUHlnWmZuQTRqXC9aenc4Rk9UOEdGU1ZSb3dvQ0pFcGI3enNIU05tdHJZNGNoRXJcL2xxT1M1aFFjbnI0ZnEzMUdWd2FaWUdcL0JUYnJpSkxUbGRtNEdcL2gzZW9WRW9sa0p5cWs4SGtDRXhPak92cTI3TyIsIm1hYyI6IjdhNDRiNjk1MDNkZTc4YjA1M2NiN2FjMjFjYmUyNGVmMjI0ZTIxZjhkYjFhNmVlODY1MzhjZmM1ZTQ4MjgyMDcifQ=="
        ));

        let form_params = [
            ("email", email.as_str()),
            ("type", "register"),
            ("sn", "525fcb21ae96167be302fef2a71606b6"),
            ("model", "Chrome/150.0.0.0"),
            ("version_code", "1"),
            ("version_name", "1.0.0"),
            ("device_name", "我的电脑"),
            ("os", "pc"),
        ];

        let response_json: Value = client.post("https://www.raccoongame.com/users/sendEmail")
            .headers(headers)
            .form(&form_params)
            .send()
            .await?
            .json()
            .await?;
    }
    anyhow::bail!("Failed to send email");
}
use std::process::Command;
use std::process::Output;

pub async fn start_vlc(current_time: &String, content_url: &String, token: Option<&String>) -> Output {
    let output: Output = Command::new("vlc")
        .arg(format!("--start-time={}", current_time))
        .arg(format!("https://audiobook.nuagemagique.duckdns.org{}?token={}", content_url, token.as_deref().unwrap_or("")))
        .output()
        .expect("Failed to execute program");
    println!("{}", current_time);
    output
}


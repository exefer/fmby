use crate::{Context, Error};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;

/// Executes a shell command on the bot's server
// Which is most likely a docker container (guaranteed in production)
#[poise::command(slash_command, owners_only, install_context = "User")]
pub async fn sh(
    ctx: Context<'_>,
    #[description = "Shell command to run"] command: String,
) -> Result<(), Error> {
    let mut command = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = command.stdout.take().unwrap();
    let stderr = command.stderr.take().unwrap();

    let output: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let stdout_task = tokio::spawn({
        let output = Arc::clone(&output);
        async move {
            let mut reader = io::BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap() > 0 {
                let mut output = output.lock().await;
                output.push(std::mem::take(&mut line));
            }
        }
    });

    let stderr_task = tokio::spawn({
        let output = Arc::clone(&output);
        async move {
            let mut reader = io::BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap() > 0 {
                let mut output = output.lock().await;
                output.push(std::mem::take(&mut line));
            }
        }
    });

    let _ = tokio::join!(stdout_task, stderr_task);

    let output = output.lock().await;
    let output = output.join("");

    ctx.say(format!("```\n{}```", output)).await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [sh()]
}

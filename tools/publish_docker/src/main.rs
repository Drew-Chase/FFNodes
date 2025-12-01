use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

#[tokio::main]
async fn main() {
    // Runs this command:
    // gh act --container-options "--privileged --user 0"
    let mut child = Command::new("gh")
        .arg("act")
        .arg("--container-options")
        .arg("--privileged --user 0")
        .args(std::env::args().skip(1))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir("./")
        .spawn()
        .expect("failed to launch gh process");

    // Take stdout and stderr before spawning tasks
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let stderr = child.stderr.take().expect("Failed to get stderr");

    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                break;
            }
            print!("{}", line);
            line.clear();
        }
    });

    let stderr_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(n) = reader.read_line(&mut line).await {
            if n == 0 {
                break;
            }
            eprint!("{}", line);
            line.clear();
        }
    });

    // Wait for both handles to complete
    let _ = tokio::join!(stdout_handle, stderr_handle);

    // Wait for the child process to complete
    let _ = child.wait().await.expect("Failed to wait for child process");
}
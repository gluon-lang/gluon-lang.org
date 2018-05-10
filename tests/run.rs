use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn test() {
    let path = PathBuf::from(::std::env::args().next().unwrap());
    let exe = path.parent()
        .and_then(|path| path.parent())
        .expect("server executable")
        .join(env!("CARGO_PKG_NAME"));
    let mut child = Command::new(exe).stdout(Stdio::piped()).spawn().unwrap();
    {
        let stdout = child.stdout.as_mut().expect("stdout missing");
        if let None = BufReader::new(stdout)
            .lines()
            .find(|line| line.as_ref().unwrap().starts_with("Server started"))
        {
            panic!("Expected the server to start");
        }
    }
    child.kill().unwrap();
}

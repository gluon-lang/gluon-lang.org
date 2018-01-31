use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn test() {
    let path = PathBuf::from(::std::env::args().next().unwrap());
    let exe = path.parent()
        .and_then(|path| path.parent())
        .expect("debugger executable")
        .join(env!("CARGO_PKG_NAME"));
    let mut child = Command::new(exe).stdout(Stdio::piped()).spawn().unwrap();
    {
        let stdout = child.stdout.as_mut().expect("stdout missing");
        let mut buffer = [0; 256];
        let n = stdout.read(&mut buffer).unwrap();
        if n == 0 {
            panic!("Unable to read anything")
        }
        if !buffer.starts_with(b"Server started") {
            panic!("Expected the server to start");
        }
    }
    child.kill().unwrap();
}

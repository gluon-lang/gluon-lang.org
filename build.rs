extern crate failure;
extern crate home;
extern crate glob;
extern crate regex;

extern crate gluon_master;

use std::{process::Command, env, path::PathBuf};

use regex::Regex;

const LOCK_FILE: &str = include_str!("Cargo.lock");

fn git_master_version() -> String {
    Regex::new("git\\+[^#]+gluon#([^\"]+)")
        .unwrap()
        .captures(LOCK_FILE)
        .expect("gluon master version")
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
}

fn gluon_git_path() -> Result<PathBuf, failure::Error> {
    let std_glob_path = home::cargo_home()?
        .join(&format!(
            "git/checkouts/gluon-*/{}",
            &git_master_version()[..7]
        ))
        .display()
        .to_string();
    Ok(glob::glob(&std_glob_path)?
        .next()
        .expect("git repo in cargo home")?)
}


fn create_docs(path: &str) -> Result<(), failure::Error> {
    let git_dir = gluon_git_path()?;

    let exit_status = Command::new("cp")
        .args(&["-r", &git_dir.join("std").to_string_lossy(), "."])
        .status()?;
    if !exit_status.success() {
        return Err(failure::err_msg("Error copying docs"));
    }

    gluon_master::generate_doc("std", path)?;

    let mut command = Command::new("mdbook");
    command.args(&[
        "build",
        "--dest-dir",
        &env::current_dir()?.join("dist/book").to_string_lossy(),
        &git_dir.join("book").to_string_lossy(),
    ]);
    println!("Building book: {:?}", command);
    let exit_status = command.status()?;
    if !exit_status.success() {
        return Err(failure::err_msg("Error building book docs"));
    }

    Ok(())
}


fn main() {
    let doc_path = "dist/doc/nightly";
    create_docs(doc_path).unwrap_or_else(|err| panic!("{}", err));
    println!("rerun-if-change=build.rs");
    println!("rerun-if-change=Cargo.lock");
}

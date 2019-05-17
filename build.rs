use env_logger;
use failure;
use glob;
use home;

use gluon_crates_io;
use gluon_master;

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
};

use regex::Regex;

const LOCK_FILE: &str = include_str!("Cargo.lock");

fn git_master_version() -> String {
    Regex::new(r#"git\+[^#]+gluon#([^"]+)"#)
        .unwrap()
        .captures(LOCK_FILE)
        .expect("gluon master version")
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
}

fn crates_io_version() -> String {
    Regex::new(r"gluon ([^ ]+) \(registry\+")
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

fn gluon_crates_io_path() -> Result<PathBuf, failure::Error> {
    let std_glob_path = home::cargo_home()?
        .join(&format!("registry/src/*/gluon-{}/", crates_io_version()))
        .display()
        .to_string();
    Ok(glob::glob(&std_glob_path)?
        .map(|result| result.unwrap())
        .max()
        .expect("crates io entry in cargo home"))
}

fn generate_doc_for_dir<P, Q, F>(
    in_dir: &P,
    out_dir: &Q,
    mut generate_doc: F,
) -> Result<(), failure::Error>
where
    P: AsRef<Path> + ?Sized,
    Q: AsRef<Path> + ?Sized,

    F: FnMut(&Path, &Path) -> Result<(), failure::Error>,
{
    generate_doc_for_dir_(in_dir.as_ref(), out_dir.as_ref(), &mut generate_doc)
}

fn generate_doc_for_dir_(
    in_dir: &Path,
    out_dir: &Path,
    generate_doc: &mut dyn FnMut(&Path, &Path) -> Result<(), failure::Error>,
) -> Result<(), failure::Error> {
    {
        eprintln!(
            "Generating gluon doc: {} -> {}",
            in_dir.display(),
            out_dir.display()
        );
        if Path::new("target/std").exists() {
            fs::remove_dir_all("target/std")?;
        }

        let before = env::current_dir()?;
        env::set_current_dir(in_dir)?;
        generate_doc(Path::new("std"), &Path::new("..").join(out_dir).join("std"))?;
        env::set_current_dir(before)?;
    }

    fs::remove_dir_all(out_dir.join("book")).or_else(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(err)
        }
    })?;
    let mut command = Command::new("mdbook");
    command.args(&[
        "build",
        "--dest-dir",
        &env::current_dir()?
            .join(out_dir)
            .join("book")
            .to_string_lossy(),
        &in_dir.join("book").to_string_lossy(),
    ]);
    eprintln!("Building book: {:?}", command);
    let exit_status = command.status()?;
    if !exit_status.success() {
        return Err(failure::err_msg("Error building book docs"));
    }
    Ok(())
}

fn create_docs() -> Result<(), failure::Error> {
    {
        let git_dir = gluon_git_path()?;
        generate_doc_for_dir(&git_dir, "dist/doc/nightly", |input, output| {
            let src_url = Some(format!(
                "https://github.com/gluon-lang/gluon/blob/{}",
                git_master_version()
            ));
            gluon_master::generate_doc(&gluon_master::gluon_doc::Options {
                input: input.to_owned(),
                output: output.to_owned(),
                src_url,
            })
        })?;
        assert!(Path::new("dist/doc/nightly/std/std.html").exists());
    }

    {
        let crates_io_dir = gluon_crates_io_path()?;
        generate_doc_for_dir(
            &crates_io_dir,
            "dist/doc/crates_io",
            gluon_crates_io::generate_doc,
        )?;
        assert!(Path::new("dist/doc/crates_io/std/std.html").exists());
    }

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(err) = create_docs() {
        eprintln!("{}\n{}", err.backtrace(), err);
        process::exit(1);
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");
}

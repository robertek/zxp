use log::{debug};
use std::process::{Command, Stdio};

pub fn xbps_update_check(root: &str) {
    debug!("runing xbps-install -M -u -r {root}");

    let mut cmd = Command::new("xbps-install")
        .arg("-M")
        .arg("-u")
        .arg("-r")
        .arg(format!("{root}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .expect("xbps failed");

    // do not update anything but --dry-run is not usable here
    let _echo = Command::new("echo")
        .arg("\"n\"")
        .stdout(cmd.stdin.take().unwrap())
        .spawn();

    // wait for the command to finish, status() may not be used when stdin is
    let _output = cmd.wait_with_output().unwrap();
}

pub fn _xbps_sync(root: &str) {
    debug!("runing xbps-install -S -r {root}");

    Command::new("xbps-install")
        .arg("-S")
        .arg("-r")
        .arg(format!("{root}"))
        .status()
        .expect("xbps failed");
}

pub fn xbps_update(root: &str, local_repo: Option<&str>) {
    if local_repo.is_some() {
        debug!("runing xbps-install -u -r {root} -R {}", local_repo.unwrap());
    } else {
        debug!("runing xbps-install -u -r {root}");
    }

    let mut binding = Command::new("xbps-install");
    let cmd = binding.arg("-u").arg("-r").arg(format!("{root}"));

    if local_repo.is_some() {
        cmd.arg("-R").arg(format!("{}", local_repo.unwrap()));
    }

    cmd.status().expect("xbps failed");
}

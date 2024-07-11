use temp_dir::TempDir;
use log::{info, error, debug};
use std::{io, fs, path};

use super::xbps::xbps_update;
use super::settings::Settings;

const GH_URL: &str = "https://api.github.com/repos";
const GH_ARTIFACTS: &str = "actions/artifacts";


fn artifacts_metadata() -> Result<serde_json::Value, String> {
    let repo = match Settings::gh_repo() {
        Ok(r) => r,
        Err(e) => return Err(e)
    };

    let url = format!("{GH_URL}/{repo}/{GH_ARTIFACTS}");
    info!("Fetching repo zip url from:\n\t{url}");

    let json: serde_json::Value = ureq::get(&url)
        .call()
        .expect("Repo artifact api failed")
        .into_json()
        .expect("Json conversion failed");

    Ok(json)
}

pub fn github_artifacts() {
    let json = match artifacts_metadata() {
        Ok(j) => j,
        Err(e) => { 
            error!("{}", e);
            return;
        }
    };

    // total_count is a number and is not expected to be high, so the unwraps would pass
    let count = usize::try_from(json["total_count"].as_number().unwrap().as_u64().unwrap()).unwrap();

    println!("Available artifacts:");
    for n in 0..count {
        let name = &json["artifacts"][n]["name"].as_str().unwrap();
        println!("\tartifact {n} = {name}");
    }
}

fn artifact_url(artifact_name: Option<String>) -> Result<String, String> {
    let json = match artifacts_metadata() {
        Ok(j) => j,
        Err(e) => return Err(e)
    };

    // total_count is a number and is not expected to be high, so the unwraps would pass
    let count = usize::try_from(json["total_count"].as_number().unwrap().as_u64().unwrap()).unwrap();

    debug!("Number of artifatcs = {count}");

    let mut index: usize = 0;

    if artifact_name.is_some() {
        let mut found: Option<usize> = None;

        for n in 0..count {
            let name = &json["artifacts"][n]["name"].as_str().unwrap();
            debug!("artifact {n} name = {name}");

            if artifact_name.as_ref().unwrap().eq(name) {
                found = Some(n);
                break;
            }
        }

        match found {
            Some(x) => { index = x },
            None => {
                return Err(format!("Artifact named \"{}\" does not exists", artifact_name.unwrap()))
            }
        }
    }

    debug!("selected artifact json response {:#?}", json["artifacts"][index]);

    // archive_download_url is a string
    Ok(json["artifacts"][index]["archive_download_url"].as_str().unwrap().to_owned())
}

fn downlad_zip(url: &str, zipfile: &str) -> Result<(), String> {
    info!("Preparing to download zip from:\n\t{url}");

    let key = match Settings::gh_key() {
        Ok(k) => k,
        Err(e) => return Err(e)
    };

    let resp = match ureq::get(&url)
        .set("Authorization", &key)
        .call() {
            Ok(r) => r,
            Err(..) => return Err(format!("Zip request failed"))
        };

    let len: usize = match resp.header("Content-Length").unwrap().parse() {
        Ok(l) => l,
        Err(..) => return Err(format!("Content-Length is missing for zipfile"))
    };

    let mut zipfp = match fs::File::create(zipfile) {
        Ok(fp) => fp,
        Err(err) => return Err(format!("Failed to create repo.zip: {err}"))
    };

    if len < 1024*1024 {
        info!("Downloading zip file (size ~{}kB)", len>>10);
    } else {
        info!("Downloading zip file (size ~{}MB)", len>>20);
    }

    match io::copy(&mut resp.into_reader(), &mut zipfp) {
        Ok(_) => return Ok(()),
        Err(e) => return Err(format!("Could not write to file: {e}"))
    };
}

fn uncompress_repo(zipfile: &str) -> Result<(), String> {
    info!("Uncompressing zip");

    let zipfp = match fs::File::open(zipfile) {
        Ok(fp) => fp,
        Err(err) => return Err(format!("Failed to open repo.zip: {err}"))
    };

    let mut archive = zip::ZipArchive::new(zipfp).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let filename = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        debug!("\t{} size={}", filename.display(), file.size());

        if file.is_dir() {
            fs::create_dir_all(&filename).unwrap();
        } else {
            if let Some(p) = filename.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = match fs::File::create(&filename) {
                Ok(fp) => fp,
                Err(e) => return Err(format!("Failed to create {}: {e}", filename.display()))
            };
            match io::copy(&mut file, &mut outfile) {
                Ok(_) => (),
                Err(e) => return Err(format!("Failed to write {}: {e}", filename.display()))
            };
        }
    }

    Ok(())
}

fn fetch_repo(url: &str, path: &str) -> Result<(), String> {
    let root = path::Path::new(path);
    match std::env::set_current_dir(&root) {
        Ok(_) => debug!("Changing working directory to {path}"),
        Err(e) => return Err(format!("Can't change directory {path}: {e}"))
    };

    let zipfile = "repo.zip";

    match downlad_zip(url, zipfile) {
        Ok(_) => (),
        Err(e) => return Err(e)
    }

    match uncompress_repo(zipfile) {
        Ok(_) => return Ok(()),
        Err(e) => return Err(e)
    }
}

/// Fetches a zipped repository from github actions artifacts.
///
/// The root variable should contain the root path on which the xbps would
/// perform the update action.
pub fn github_update(root: &str, artifact_name: Option<String>) {
    // Create a temporary directory for the repository
    let local_repo = match TempDir::new() {
        Ok(t) => t,
        Err(..) => {
            error!("Failed to create temporary repository directory.");
            return;
        }
    };

    let repo_path = local_repo.path().to_str().unwrap().to_owned();
    //let repo_path = "/tmp/test".to_string();
    debug!("Created temporary repository in {}", repo_path);

    // Get the URL of the artifact zipfile
    let zip_url = match artifact_url(artifact_name) {
        Ok(u) => u,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    // Download the zipfile and unpack it
    match fetch_repo(&zip_url, &repo_path) {
        Ok(_) => {},
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    // TODO: implement zip checksum verification

    // Perform xbps update on the temporary repository
    xbps_update(root, Some(&repo_path));

    // The cleanup of temporary directory is done by the TempDir crate when the
    // handle lose its lifetime.
    info!("Cleaning up the tmp directory {}", repo_path);
}

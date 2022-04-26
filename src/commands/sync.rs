use std::{process::{Command, Stdio}, error::Error, io::{BufReader, Cursor, BufRead}, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use repo_icons::RepoIcons;
use site_icons::IconInfo;
use tokio::{task::JoinHandle, fs::File, io::copy};
use futures::future;
use diesel::RunQueryDsl;
use crate::{database::{self, db},modify_gitignore, get_slug, CACHE_DIR, GitIcons, models::{Icon, Repo}};

pub async fn sync_all() -> Result<(), Box<dyn Error>> {
  let home_dir = home::home_dir().unwrap();
  let home_dir = home_dir.to_str().unwrap();

  let mut cmd = Command::new("find")
    .args([
      home_dir,
      "-path",
      &format!("{}/.Trash", home_dir),
      "-prune",
      "-o",
      "-path",
      &format!("{}/Library", home_dir),
      "-prune",
      "-o",
      "-type",
      "d",
      "-name",
      ".git",
      "-exec",
      "echo",
      "{}",
      ";",
    ])
    .stderr(Stdio::inherit())
    .stdout(Stdio::piped())
    .spawn()?;

  let stdout = cmd.stdout.as_mut().unwrap();
  let stdout_reader = BufReader::new(stdout);
  let stdout_lines = stdout_reader.lines();

  for repo_path in stdout_lines {
    let repo_path = repo_path?;
    let repo_path = match repo_path.strip_suffix("/.git") {
      Some(repo_path) => repo_path,
      None => &repo_path,
    };

    // ignore paths contained within hidden folders
    if repo_path
      .split("/")
      .find(|path| path.starts_with("."))
      .is_some()
    {
      continue;
    }

    println!("{}", &repo_path);

    match sync(&repo_path).await {
      Err(error) => eprintln!("{}", error),
      _ => {}
    };
  }
  // let tasks: Vec<_> = stdout_lines
  //   .map(|repo_path| {
  //     tokio::spawn(async move {
  //       GitIcons::sync(&repo_path.unwrap()).await.unwrap();
  //     })
  //   })
  //   .collect();

  // future::join_all(tasks).await;

  Ok(())
}

pub async fn sync(slug_or_path: &str) -> Result<(), Box<dyn Error>> {
  modify_gitignore::modify()?;

  let (user, repo_name, repo_path) = get_slug(slug_or_path)?;
  let icons = RepoIcons::load(&user, &repo_name).await;

  if let Ok(icons) = icons {
    let mut tasks: Vec<_> = icons
      .into_iter()
      .enumerate()
      .map(|(i, icon)| -> JoinHandle<Option<()>> {
        let slug_or_path = slug_or_path.to_string();
        let (user, repo_name) = (user.clone(), repo_name.clone());

        tokio::spawn(async move {
          let cache_name = format!("{}{}", icon.url.host_str().unwrap_or(""), icon.url.path())
            .replace("/", "-")
            .replace(":", "-");

          let mut hasher = DefaultHasher::new();
          cache_name.hash(&mut hasher);

          let icon_name = format!(
            "{}.{}",
            hasher.finish().to_string(),
            match icon.info {
              IconInfo::PNG { .. } => "png",
              IconInfo::JPEG { .. } => "jpg",
              IconInfo::ICO { .. } => "ico",
              IconInfo::SVG => "svg",
            }
          );
          let icon_path = CACHE_DIR.join(icon_name.clone());

          if !icon_path.exists() {
            let mut icon_file = File::create(&icon_path).await.ok()?;

            match icon.url.scheme() {
              "data" => {
                let data_uri_path = icon.url.path();
                let data_index = data_uri_path.find(",").unwrap_or(0);
                let type_index = data_uri_path[..data_index].find(";");

                let data = data_uri_path[(data_index + 1)..].to_string();
                let mut written = false;

                if let Some(type_index) = type_index {
                  let data_type = data_uri_path[(type_index + 1)..data_index].to_string();
                  if data_type == "base64" {
                    let mut content = Cursor::new(base64::decode(&data).unwrap_or(Vec::new()));
                    copy(&mut content, &mut icon_file).await.ok()?;
                    written = true;
                  }
                }

                if !written {
                  let mut content = Cursor::new(urlencoding::decode_binary(&data.as_bytes()));
                  copy(&mut content, &mut icon_file).await.ok()?;
                }
              }
              _ => {
                let response = reqwest::get(icon.url).await.ok()?;
                let mut content = Cursor::new(response.bytes().await.ok()?);

                copy(&mut content, &mut icon_file).await.ok()?;
              }
            }
          }

          // If it's the first icon, then write it as the default to
          if i == 0 {
            GitIcons::set(&slug_or_path, &icon_name, false).await.ok()?;
          }

          let icon = Icon {
            owner: user,
            repo: repo_name,
            path: icon_name,
          };

          {
            use database::schema::icons::dsl::*;
            diesel::insert_or_ignore_into(icons)
              .values(&icon)
              .execute(db())
              .ok()?;
          }

          Some(())
        })
      })
      .collect();

    while !tasks.is_empty() {
      match future::select_all(tasks).await {
        (Ok(_), _index, remaining) => {
          tasks = remaining;
        }
        (Err(error), _index, remaining) => {
          eprintln!("{:?}", error);
          tasks = remaining;
        }
      }
    }
  } else {
    eprintln!("{:?}", icons);

    // add the repo with an empty icon
    if let Some(repo_path) = repo_path {
      let new_repo = Repo {
        owner: user.clone(),
        repo: repo_name.clone(),
        path: repo_path.clone(),
        icon_path: None,
      };

      {
        use database::schema::repos::dsl::*;

        diesel::insert_or_ignore_into(repos)
          .values(&new_repo)
          .execute(db())?;
      }
    }
  }

  GitIcons::write(&slug_or_path).await?;

  Ok(())
}
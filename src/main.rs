use std::{fmt::Display, path::PathBuf};

use clap::Parser;
use regex::Regex;
use tokio::{
    fs,
    io::{self, AsyncWriteExt},
};

const FILE_NAME: &str = "flypy_user.txt";
const BACK_FILE: &str = "flypy_user.txt.back";
const GIT_FILE_URL: &str = "https://github.com/FengYouJun520/flypy_user/blob/main/flypy_user.txt";

struct UserDict {
    pub name: String,
    pub simple: String,
}

impl UserDict {
    fn new(name: String, simple: String) -> Self {
        Self { name, simple }
    }
}

impl Display for UserDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.name, self.simple)
    }
}

/// 命令行配置
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Config {
    /// 用户的配置目录
    #[arg(short, long = "config", value_name = "PATH")]
    config_dir: Option<PathBuf>,
    /// git仓库的flypy_user.txt路径
    git_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let mut config_dir = if let Some(ref dir) = config.config_dir {
        dir.clone()
    } else {
        dirs::config_dir().unwrap()
    };
    config_dir.push("Rime");

    let resp = reqwest::get(&config.git_url).await?;
    if resp.status() != 200 {
        println!("请求失败: {}，请检查路径是否正确", config.git_url);
        println!("例如: {}", GIT_FILE_URL);
        std::process::exit(1);
    }

    let text = resp.text().await?;
    let re = Regex::new(
        r#"<td id="LC\d+" class="blob-code blob-code-inner js-file-line">(?P<name>.*)\s+(?P<simple>.*)</td>"#,
    )?;

    let mut user_dicts = vec![];
    for m in re.captures_iter(&text) {
        user_dicts.push(UserDict::new(m["name"].to_owned(), m["simple"].to_owned()));
    }

    let mut back_path = config_dir.clone();
    back_path.push(BACK_FILE);

    let mut flypy_dir = config_dir.clone();
    flypy_dir.push(FILE_NAME);

    // 备份文件
    if !back_path.exists() {
        fs::copy(&flypy_dir, back_path).await?;
    }

    let file = fs::OpenOptions::new().append(true).open(&flypy_dir).await?;
    // 导入数据
    let mut buf = io::BufWriter::new(file);
    buf.write("\n\n用户自定义词库\n".as_bytes()).await?;
    for user_dict in user_dicts.iter() {
        buf.write(format!("{}\n", user_dict).as_bytes()).await?;
        println!("导入: {}", user_dict);
    }

    buf.flush().await?;

    println!("导入完成, 共导入: {}个词", user_dicts.len());

    open::that(&config_dir)?;
    Ok(())
}

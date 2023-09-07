use std::{env, fs};
use std::fs::File;
use reqwest;
use scraper::{Html, Selector};
use regex::Regex;
use std::process::{Command, Stdio};
use reqwest::get;
use super::tui_lib::tui_print;

fn get_yt_page(req: &str) -> Result<String, Box<dyn std::error::Error>> {

    let yt_page_txt = reqwest::blocking::get(String::from("https://www.youtube.com/results?search_query=")+req)?
        .text()?;
    return Ok(yt_page_txt);
}

pub fn research_on_yt(research_txt: &str) -> Vec<String>{

    let mut research_results: Vec<String> = Vec::new();
    let yt_page_txt = get_yt_page(research_txt).unwrap();
    let document = Html::parse_document(&yt_page_txt);
    let scripts = Selector::parse("script").unwrap();

    for script in document.select(&scripts) {

        let script_txt = script.text().collect::<Vec<_>>();
        if script_txt.len() > 0 {

            match script_txt[0].find("watch?v=") {
                Some(_) => {

                    let re = Regex::new(r#"watch\?v=.*?""#).unwrap();
                    for matching_pat in re.find_iter(script_txt[0]) {
                        let url = matching_pat.as_str().split("\\").collect::<Vec<&str>>()[0];
                        research_results.push(String::from(url));
                    }

                },
                None => {},
            }
        }

    }
    return research_results;
}

fn exec_command(parts_path_exec: &Vec<&str>, args_exec: &Vec<&str>){
    let mut exe_path = env::current_exe().expect("Failed to get executable path");
    exe_path.pop(); // Remove the executable name from the path
    for part in parts_path_exec {
        exe_path.push(part);
    }

    let mut command = Command::new(&exe_path);
    for arg_exec in args_exec {
        command.arg(arg_exec);
    }

     // println!("Executing command: {:?}", command);
    tui_print("Start Downloading");

    let output = command
        .stdout(Stdio::inherit())
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let _stdout = String::from_utf8_lossy(&output.stdout);
        // println!("Command executed successfully:\n{}", _stdout);
        tui_print("Download done");
    } else {
        let _stderr = String::from_utf8_lossy(&output.stderr);
        // println!("Command failed:\n{}", _stderr);
    }
}

pub fn exec_command_yt_dl(url: &str, file_name: &str) {
    let mut data_path = env::current_exe().expect("Failed to get executable path");
    data_path.pop(); // Remove the executable name from the path
    data_path.push("data");
    data_path.push(format!("{}.%(ext)s", file_name));
    // println!("{:?}", data_path);
    exec_command(
        &vec![
            "utils",
            "yt-dlp"
        ],
        &vec![
            "-vU",
            "-x",
            "--audio-format",
            "wav",
            //"mp3",
            //"--audio-quality",
            //"192K",
            "-q",
            "--progress",
            "-o",
            &data_path.to_string_lossy().to_string(),
            &format!("https://www.youtube.com/{}", url)
        ]
    );

}

pub fn dependencies_check() -> bool{
    let folder_path = "utils";
    #[cfg(not(target_os = "windows"))]
    let file_names = vec!["yt-dlp", "ffmpeg"];
    #[cfg(target_os = "windows")]
        let file_names = vec!["yt-dlp", "ffmpeg.exe"];
    let mut path = env::current_exe().expect("Failed to get executable path");
    path.pop();
    path.push(folder_path);

    for file_name in file_names{
        let mut path_clone = path.clone();
        path_clone.push(file_name);
        let file_path = path_clone.to_string_lossy().to_string();

        if fs::metadata(&file_path).is_ok() {
            println!("The file {} exists in the {} folder.", file_name, folder_path);
        } else {
            println!("The file {} does not exist in the {} folder.", file_name, folder_path);
            return false;
        }
    }
    return true;
}

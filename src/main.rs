mod lib;

use std::{env, fs, io};
use std::sync::{Arc, Mutex};
use rodio::Sink;
use std::thread::sleep;
use std::time::Duration;
use lib::tui_lib::{ask_prompt, display_menu, tui_print};
use lib::model_lib::{AudioPlayer, Menu};


fn init()-> io::Result<()>{
    // Directory where the files are located
    let mut dir_path = env::current_exe().expect("Failed to get executable path");
    dir_path.pop();
    dir_path.push("data");


    let entries = fs::read_dir(dir_path.to_string_lossy().to_string())?;

    for entry in entries {
        let entry = entry?;

        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.starts_with("test") && file_name.ends_with(".wav") {
                let file_path = entry.path();

                if file_path.is_file() {
                    fs::remove_file(file_path.clone())?;
                    println!("Removed file: {:?}", file_path);
                }
            }
        }
    }

    Ok(())

}

fn playlist_menu() -> i32{
    let playlist_menu = Menu::playlist_menu();

    let playlist_menu_input_choice = display_menu(
        &playlist_menu.menu_choice,
        &playlist_menu.menu_choice_quit,
        false
    );
    match playlist_menu_input_choice {
        5 => println!("*********"),
        -1 => println!("bye!!!"),
        _ => println!("*****ssssss****"),
    }
    return playlist_menu_input_choice
}

fn main_menu(ap_origin: &mut Arc<Mutex<AudioPlayer>>){
    let binding_ap = ap_origin.clone();
    let mut ap = binding_ap.lock().unwrap();
    let main_menu = Menu::main_menu();

    let main_menu_input_choice = display_menu(
        &main_menu.menu_choice,
        &main_menu.menu_choice_quit,
        true
    );

    match main_menu_input_choice {
        1 => {
            let title_research = ask_prompt("title of your research: ");
            ap.add_audio(&title_research, );
        },
        2 =>{
            if ap.is_paused() {
                ap.resume();
            }else {
                ap.pause();
            }
        },
        3 =>{
            ap.next_audio();
        },
        4 =>{
          ap.print_audio_list();
        },
        5 => {
            loop {
                let loop_status = playlist_menu();
                if loop_status == -1 {
                    break;
                }
                sleep(Duration::from_millis(1000));
            }
        },
        6 => {
            tui_print("Author: K-A-R-I-M \nhttps://github.com/K-A-R-I-M");
        },
        7 => {
            tui_print("No Parameter for now!!!");
        },
        _ => {},
    }
}



fn main() {

    let _ = init();

    //--------------------------------- AUDIO INIT ---------------------------------
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let audio_player = Arc::new(Mutex::new(sink));
    //--------------------------------- AUDIO PLAYER INIT ---------------------------------
    let mut ap = Arc::new(Mutex::new(AudioPlayer::new(audio_player)));


    //--------------------------------- MEDIA CONTROL INTERNAL INIT ---------------------------------


    loop {
        main_menu(&mut ap);
        sleep(Duration::from_millis(1000));
    }

}
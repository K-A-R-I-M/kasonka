mod lib;

use std::{env, fs, io, thread};
use std::ffi::c_void;
use std::process::exit;
use std::sync::{Arc, Mutex};
use rodio::Sink;
use std::thread::sleep;
use std::time::Duration;
use std::ptr::null_mut;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::Foundation::HWND;
use crate::lib::central_lib::dependencies_check;
use crate::lib::tui_lib::{ask_prompt, display_menu, GeneralSignal, tui_print};
use crate::lib::model_lib::{AudioPlayer, AudioPlayerStatus, GeneralVars, MediaControlsInternal, Menu, PlaylistKa};


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

fn playlist_menu(cont_pk_origin: &mut Arc<Mutex<Vec<PlaylistKa>>>) -> GeneralSignal {

    let binding_cont_pk = cont_pk_origin.clone();
    let mut cont_pk = binding_cont_pk.lock().unwrap();
    let playlist_menu = Menu::playlist_menu();

    let playlist_menu_input_choice = display_menu(
        &playlist_menu.menu_choice,
        &playlist_menu.menu_choice_quit
    );
    match playlist_menu_input_choice {
        GeneralSignal::ValidInput(1) => {
            let title_playlist = ask_prompt("playlist name: ");
            cont_pk.push(PlaylistKa::new(title_playlist, Vec::new()));
        },
        GeneralSignal::ValidInput(2) => {
            for pk in &*cont_pk{
                println!("{}", pk.title);
            }
        },
        GeneralSignal::ValidInput(3) => {
            let id_playlist_str = ask_prompt("playlist id: ");

            // Use the `parse()` method to attempt conversion
            let id_playlist_int = id_playlist_str.replace("\n", "").parse::<i32>();

            // Check if parsing was successful
            match id_playlist_int {
                Ok(id_playlist) => {
                    if (id_playlist as u32) < ((*cont_pk).len() as u32) && id_playlist >= 0 {
                        for ak in &(&*cont_pk.get(id_playlist as usize).unwrap()).audios{
                            println!("{}", ak.title);
                        }
                    }
                }
                Err(_) => {
                    println!("{:?}", id_playlist_str);
                    println!("{:?}", id_playlist_int);
                    println!("Failed to parse the string as an integer.");
                }
            }

        },
        GeneralSignal::ValidInput(4) => {

        },
        GeneralSignal::ValidInput(5) => println!("*********"),
        GeneralSignal::Exit => println!("bye!!!"),
        _ => println!("*****ssssss****"),
    }
    return playlist_menu_input_choice
}

fn main_menu(ap_origin: &mut Arc<Mutex<Option<AudioPlayer>>>, cont_pk_origin: &mut Arc<Mutex<Vec<PlaylistKa>>>) -> GeneralSignal {
    let mut app_status = GeneralSignal::Nothing;

    let ap_local_clone = ap_origin.clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

    let mut cont_pk_local_clone = cont_pk_origin.clone();


    let main_menu = Menu::main_menu();

    if matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Disabled){
        app_status = GeneralSignal::Exit;
        return app_status;
    }

    let main_menu_input_choice = display_menu(
        &main_menu.menu_choice,
        &main_menu.menu_choice_quit
    );

    match main_menu_input_choice {
        GeneralSignal::ValidInput(1) => {
            let title_research = ask_prompt("title of your research: ");
            ap.add_audio(&title_research);
        },
        GeneralSignal::ValidInput(2) => {
            if ap.is_paused() {
                ap.resume();
            }else {
                ap.pause();
            }
        },
        GeneralSignal::ValidInput(3) => {
            ap.next_audio();
        },
        GeneralSignal::ValidInput(4) => {
          ap.print_audio_list();
        },
        GeneralSignal::ValidInput(5) => {
            loop {
                let loop_status = playlist_menu(&mut cont_pk_local_clone);
                if matches!(loop_status, GeneralSignal::Exit) {
                    break;
                }
                sleep(Duration::from_millis(1000));
            }
        },
        GeneralSignal::ValidInput(6) => {
            app_status = GeneralSignal::Reboot;
        },
        GeneralSignal::ValidInput(7) => {
            tui_print("Author: K-A-R-I-M \nhttps://github.com/K-A-R-I-M");
        },
        GeneralSignal::ValidInput(8) => {
            tui_print("No Parameter for now!!!");
        },
        GeneralSignal::InvalidInput =>{
            println!("Invalid input");
        }
        GeneralSignal::Exit => {
            app_status = GeneralSignal::Exit;
        }
        _ => {},
    }
    return app_status;
}

fn main_thread(mut gv_main_thread: GeneralVars) -> GeneralSignal{
    let _ = init();
    let mut app_status: GeneralSignal;

    let mut cont_pk: Arc<Mutex<Vec<PlaylistKa>>> = gv_main_thread.c_pk;
    let mut ap = gv_main_thread.ap;


    //--------------------------------- MEDIA CONTROL INTERNAL INIT ---------------------------------

    loop {
        app_status = main_menu(&mut ap, &mut cont_pk);
        if matches!(app_status, GeneralSignal::Exit) || matches!(app_status, GeneralSignal::Reboot) {
            break;
        }
        sleep(Duration::from_millis(1000));
    }


    //------------------------------ reboot audio system -----------------------------------
    //--------------------------------- AUDIO INIT ---------------------------------
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let audio_player = Arc::new(Mutex::new(sink));
    //--------------------------------- AUDIO PLAYER INIT ---------------------------------
    gv_main_thread.ap = Arc::new(Mutex::new(Some(AudioPlayer::new(audio_player))));

    gv_main_thread.c_pk = Arc::new(Mutex::new(Vec::new()));


    return app_status;
}

fn main() {

    let deps_status = dependencies_check();

    if deps_status {

        let mut gv = GeneralVars::new();

        let mut controls = Arc::new(Mutex::new(None));


        //--------------------------------- AUDIO INIT ---------------------------------
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let audio_player = Arc::new(Mutex::new(sink));
        //--------------------------------- AUDIO PLAYER INIT ---------------------------------
        let ap = Arc::new(Mutex::new(Some(AudioPlayer::new(audio_player))));

        //--------------------------------- CONT PlaylistKa INIT ---------------------------------
        let cont_pk: Arc<Mutex<Vec<PlaylistKa>>> = Arc::new(Mutex::new(Vec::new()));


        //--------------------------------- MediaControlInternal INIT ---------------------------------
        #[cfg(not(target_os = "windows"))]
            let hwnd = None;

        #[cfg(target_os = "windows")]
            let mut hwnd = {
                let mut re_hwnd = None;
                let mut raw_hwnd = unsafe { GetConsoleWindow() };
                match raw_hwnd.0 {
                    0 => println!("Error getting console window handle"),
                    pre_hwnd => {
                        //println!("Console window handle: {:?}", hwnd)
                        re_hwnd = Some(pre_hwnd as *mut c_void);
                    },
                }
                re_hwnd
            };
    if (cfg!(target_os="windows") && !(matches!(hwnd, None))) || !(cfg!(target_os="windows")) {
        let config = PlatformConfig {
            dbus_name: "my_player",
            display_name: "My Player",
            hwnd: hwnd,
        };
        match MediaControls::new(config) {
            Ok(mc) => {
                controls = Arc::new(Mutex::new(Some(MediaControlsInternal::new(mc))));
            }
            Err(error) => {
                println!("no media available {:?}", error);
                println!("{:?}", hwnd);
            }
        }

    }


        gv.ap = ap;
        gv.c_pk = cont_pk;
        gv.mci = controls;

        loop {
            let gv_thread_main = gv.clone();
            let gv_thread_player_check = gv.clone();
            let mut gv_thread_media_controls = gv.clone();

            let _thread_player_check = thread::spawn(move|| {
                AudioPlayer::start_auto_next(gv_thread_player_check.ap.clone(), gv_thread_player_check.mci.clone());
            });

            let _thread_media_controls = thread::spawn(move|| {
                let local_mci = gv_thread_media_controls.mci.lock().unwrap();
                if !(local_mci.is_none()) {
                    drop(local_mci);
                    MediaControlsInternal::attach_os_notify(&mut gv_thread_media_controls.ap.clone(), &mut gv_thread_media_controls.mci.clone());
                }
            });

            let thread_main_join = thread::spawn(move|| {
                main_thread(gv_thread_main)
            });

            let app_status = thread_main_join.join().unwrap();

            if matches!(app_status, GeneralSignal::Exit) {
                exit(0);
            }
            tui_print("Reboot Kasonka");
        }
    }else {
        println!("error with dependencies");
    }
}
use std::{env, thread};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use rodio::{Sink, Source};
use super::central_lib::{exec_command_yt_dl, research_on_yt};
use super::tui_lib::tui_print;

pub struct Menu{
    pub menu_choice: Vec<String>,
    pub menu_choice_quit : Vec<String>,
}

pub enum AudioPlayerStatus{
    Pause,
    Play,
    Empty
}

pub struct AudioPlayer {
    pub status: AudioPlayerStatus,
    pub current_nb_audios: u32,
    pub nb_audios: u32,
    pub play_obj: Arc<Mutex<Sink>>,
    pub list_audio: Vec<String>,
}

// pub struct PlaylistKa{
//     pub titre: String,
//     pub audios: Vec<String>,
// }

impl Menu {
    pub fn main_menu() -> Self{
        Self {
            menu_choice: vec![
                String::from("jouer un audio"),
                String::from("pause/reprendre l'audio"),
                String::from("next audio"),
                String::from("afficher le file de lecture"),
                String::from("menu playlist"),
                String::from("credits"),
                String::from("parametre"),
            ],
            menu_choice_quit: vec![
                String::from("sortir"),
            ]
        }
    }
    pub fn playlist_menu() -> Self{
        Self {
            menu_choice: vec![
                String::from("crée une playlist"),
                String::from("afficher tout les playlists existantes"),
                String::from("afficher une playlist"),
                String::from("gerer une playlist"),
                String::from("jouer une playlist"),
                String::from("importer une playliste youtube"),
                String::from("skip la playlist courante"),
            ],
            menu_choice_quit: vec![
                String::from("retour"),
            ]
        }
    }
}


impl AudioPlayer{
    pub fn new(audio_player: Arc<Mutex<Sink>>) -> Self{
        Self{
            status: AudioPlayerStatus::Empty,
            current_nb_audios: 0,
            nb_audios : 0,
            play_obj: audio_player,
            list_audio: Vec::new(),
        }
    }

    pub fn add_audio(&mut self, title: &str){
        let string_title = title.clone();
        self.nb_audios = self.nb_audios + 1;
        self.list_audio.push(string_title.to_string());

        self.research_download(Some(string_title.to_string()));

        // init
        if  self.nb_audios == 1 {
            self.status = AudioPlayerStatus::Play;
            self.current_nb_audios = self.current_nb_audios + 1;
            self.play_audio();
        }

        tui_print(format!("Add to audios list : {}", string_title.to_string()).as_str());
    }

    pub fn pause(&mut self){
        self.play_obj.lock().unwrap().pause();
        self.status = AudioPlayerStatus::Pause;
    }

    pub fn resume(&mut self){
        self.play_obj.lock().unwrap().play();
        self.status = AudioPlayerStatus::Play;
    }

    pub fn is_paused(&mut self) -> bool{
        return matches!(self.status, AudioPlayerStatus::Pause);
    }

    pub fn print_audio_list(&mut self){
        for audio in &self.list_audio {
            println!("{}", audio);
        }
    }

    pub fn next_audio(&mut self){
        if !(matches!(self.status, AudioPlayerStatus::Empty)){
            self.play_obj.lock().unwrap().stop();
            self.play_obj.lock().unwrap().clear();

            if self.current_nb_audios < self.nb_audios{
                self.current_nb_audios = self.current_nb_audios + 1;
            }

            self.play_audio();
            self.status = AudioPlayerStatus::Play;
        }
    }

    fn research_download(&mut self, input_research: Option<String>) -> String{
        // Research on youtube
        tui_print(format!("Researching : {}", input_research.clone().unwrap()).as_str());
        let results = research_on_yt(&input_research.clone().unwrap());

        // Download and create file
        // println!("Executing command !! ");
        tui_print(format!("Downloading : {}", input_research.clone().unwrap()).as_str());
        let file_name = format!("test{}", self.nb_audios);
        exec_command_yt_dl(&results[0], &file_name);

        return file_name;
    }

    pub fn play_audio(&mut self){
        // VAR
        let audio_player_clone = self.play_obj.clone();
        let file_name = format!("test{}", self.current_nb_audios);

        // Get audio file path  which is clean
        let mut path = env::current_exe().expect("Failed to get executable path");
        path.pop();
        path.push("data");

        // Spawn a thread to play the audio
        let current_audio_title= self.list_audio.clone().get((self.current_nb_audios.clone() - 1) as usize).map_or_else(||String::from("None"), |inner_value| inner_value.to_string());
        tui_print(format!("Start playing : {}", current_audio_title).as_str());
        thread::spawn(move || {
            let mut audio_player = audio_player_clone.lock().unwrap();
            Self::exec_play_audio(&path.to_string_lossy().to_string(), &mut audio_player, &file_name);

        });

    }

    fn exec_play_audio(dir_path: &str, audio_player: &Sink, file_name: &str){
        // println!("{:?}", format!("{}\\{}.mp3", dir_path, file_name));
        // Open the audio file
        let file = File::open(format!("{}\\{}.wav", dir_path, file_name)).unwrap();
        let audio_source = rodio::Decoder::new(BufReader::new(file)).unwrap();


        println!("controls updqte");
        println!("{:?}", audio_source.total_duration());

        // Play the audio file
        audio_player.append(audio_source);
        audio_player.play();
    }
}

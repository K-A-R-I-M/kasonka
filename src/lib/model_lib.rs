use std::{env, thread};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use rodio::{Sink, Source};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use super::central_lib::{exec_command_yt_dl, research_on_yt};
use super::tui_lib::tui_print;

#[derive(Clone)]
pub struct GeneralVars {
    pub ap: Arc<Mutex<Option<AudioPlayer>>>,
    pub c_pk: Arc<Mutex<Vec<PlaylistKa>>>,
    pub mci: Arc<Mutex<Option<MediaControlsInternal>>>,
}

impl GeneralVars {
    pub fn new()->Self{
        Self{
            ap: Arc::new(Mutex::new(None)),
            c_pk: Arc::new(Mutex::new(Vec::new())),
            mci: Arc::new(Mutex::new(None)),
        }
    }
}



pub struct Menu{
    pub menu_choice: Vec<String>,
    pub menu_choice_quit : Vec<String>,
}

impl Menu {
    pub fn main_menu() -> Self{
        Self {
            menu_choice: vec![
                String::from("jouer un audio"),
                String::from("pause/reprendre l'audio"),
                String::from("next audio"),
                String::from("afficher le file de lecture"),
                String::from("menu playlist"),
                String::from("arreter/redemarrer le systeme de lecture audio"),
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



#[derive(Clone)]
pub enum AudioPlayerStatus{
    Pause,
    Play,
    Empty,
    Disabled,
}

#[derive(Clone)]
pub struct AudioPlayer {
    pub status: Arc<Mutex<AudioPlayerStatus>>,
    pub current_nb_audios: Arc<Mutex<u32>>,
    pub nb_audios: Arc<Mutex<u32>>,
    pub play_obj: Arc<Mutex<Sink>>,
    pub list_audio: Vec<PlaylistKa>,
    pub queue_audio: Vec<PlaylistKa>,
    pub thread_status: Arc<Mutex<bool>>,
    pub current_audio_time: Arc<Mutex<Duration>>,
}

impl AudioPlayer{
    pub fn new(audio_player: Arc<Mutex<Sink>>) -> Self{
        Self{
            status: Arc::new(Mutex::new(AudioPlayerStatus::Empty)),
            current_nb_audios: Arc::new(Mutex::new(0)),
            nb_audios : Arc::new(Mutex::new(0)),
            play_obj: audio_player,
            list_audio: Vec::new(),
            queue_audio: Vec::new(),
            thread_status: Arc::new(Mutex::new(true)),
            current_audio_time: Arc::new(Mutex::new(Duration::new(0, 0))),
        }
    }

    pub fn new_none()->Self{
        Self{
            status: Arc::new(Mutex::new(AudioPlayerStatus::Disabled)),
            current_nb_audios: Arc::new(Mutex::new(0)),
            nb_audios : Arc::new(Mutex::new(0)),
            play_obj: Arc::new(Mutex::new(Sink::new_idle().0)),
            list_audio: Vec::new(),
            queue_audio: Vec::new(),
            thread_status: Arc::new(Mutex::new(true)),
            current_audio_time: Arc::new(Mutex::new(Duration::new(0, 0))),
        }
    }

    pub fn start_auto_next(ap_raw: Arc<Mutex<Option<AudioPlayer>>>, mci_raw: Arc<Mutex<Option<MediaControlsInternal>>>){
        let ap_clone = ap_raw.clone();
        let mut binding_ap = ap_clone.lock().unwrap();
        let mut binding_ap_none = AudioPlayer::new_none();
        let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

        let mci_clone = mci_raw.clone();
        let mut binding_mci = mci_clone.lock().unwrap();

        if !(matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Disabled)) {
            let ap_clone_thread = ap.clone();
            let ap_clone_thread_2 = ap.clone();

            drop(binding_ap);

            thread::spawn(move || {
                let mut ap = ap_clone_thread;

                loop {
                    if *ap.thread_status.lock().unwrap() {
                        if ap.is_nextable() {
                            while !(ap.play_obj.lock().unwrap().empty()) {
                                sleep(Duration::new(1, 0));
                            }
                            ap.next_audio();
                        }
                        sleep(Duration::new(1, 0));
                    } else {
                        break;
                    }
                }
            });



            if !(binding_mci.as_mut().is_none()) {
                thread::spawn(move || {
                    let mci_local_clone = mci_raw.clone();
                    let mut mci_local = mci_local_clone.lock().unwrap();
                    let mut ap = ap_clone_thread_2;

                    loop {
                        if *ap.thread_status.lock().unwrap() {
                            match *ap.status.lock().unwrap() {
                                AudioPlayerStatus::Play => {
                                    mci_local.as_mut().unwrap().media_controls.set_playback(MediaPlayback::Playing { progress: None }).expect("set playing : panic message");
                                },
                                AudioPlayerStatus::Pause => {
                                    mci_local.as_mut().unwrap().media_controls.set_playback(MediaPlayback::Paused { progress: None }).expect("set pause : panic message");
                                },
                                _ => {

                                }
                            }
                            sleep(Duration::new(1, 0));
                        } else {
                            break;
                        }
                    }
                });
            }
        }
    }

    pub fn add_audio(&mut self, title: &str){
        let string_title = title.clone();
        let nb_audio_clone = self.nb_audios.clone();
        let current_nb_audios_clone = self.current_nb_audios.clone();

        let nb_audio_clone_u32 = *nb_audio_clone.lock().unwrap();
        *nb_audio_clone.lock().unwrap() = nb_audio_clone_u32 + 1;

        let pk_current_audio = PlaylistKa::new(string_title.to_string(), vec!(AudioKa::new_simple(string_title.to_string())));
        self.list_audio.push(pk_current_audio.clone());
        self.queue_audio.push(pk_current_audio.clone());

        self.research_download(Some(string_title.to_string()));

        // init
        if  *nb_audio_clone.lock().unwrap() == 1 {
            *self.status.lock().unwrap() = AudioPlayerStatus::Play;

            let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();

            *current_nb_audios_clone.lock().unwrap() = current_nb_audios_clone_u32 + 1;
            self.play_audio();
        }

        tui_print(format!("Add to audios list : {}", string_title.to_string()).as_str());
    }

    pub fn pause(&mut self){
        self.play_obj.lock().unwrap().pause();
        *self.status.lock().unwrap() = AudioPlayerStatus::Pause;
    }

    pub fn resume(&mut self){
        self.play_obj.lock().unwrap().play();
        *self.status.lock().unwrap() = AudioPlayerStatus::Play;
    }

    pub fn is_paused(&mut self) -> bool{
        return matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Pause);
    }

    pub fn is_nextable(&mut self) -> bool{
        let nb_audio_clone = self.nb_audios.clone();
        let current_nb_audios_clone = self.current_nb_audios.clone();
        let nb_audio_clone_u32 = *nb_audio_clone.lock().unwrap();
        let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();

        return current_nb_audios_clone_u32 < nb_audio_clone_u32 && nb_audio_clone_u32 > 1;
    }

    pub fn print_audio_list(&mut self){
        for playlistka in &self.list_audio {
            for audioka in &playlistka.audios{
                println!("{}", audioka.title);
            }
        }
    }

    pub fn next_audio(&mut self){
        if !(matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Empty)){
            self.play_obj.lock().unwrap().stop();

            let current_nb_audios_clone = self.current_nb_audios.clone();

            if self.is_nextable() {
                let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();
                *self.current_nb_audios.lock().unwrap() = current_nb_audios_clone_u32 + 1;
            }
            self.play_audio();
            *self.status.lock().unwrap() = AudioPlayerStatus::Play;
        }
    }

    fn research_download(&mut self, input_research: Option<String>) -> String{
        // Research on youtube
        tui_print(format!("Researching : {}", input_research.clone().unwrap()).as_str());
        let results = research_on_yt(&input_research.clone().unwrap());

        // Download and create file
        // println!("Executing command !! ");
        tui_print(format!("Downloading : {}", input_research.clone().unwrap()).as_str());
        let file_name = format!("test{}", *self.nb_audios.lock().unwrap());
        exec_command_yt_dl(&results[0], &file_name);

        return file_name;
    }

    pub fn play_audio(&mut self){
        // VAR

        let audio_player_clone = self.play_obj.clone();
        let current_audio_time_clone = self.current_audio_time.clone();
        let _list_audio_clone = self.list_audio.clone();
        let _current_nb_audios_clone_u32 = *self.current_nb_audios.clone().lock().unwrap();
        let file_name = format!("test{}", *self.current_nb_audios.lock().unwrap());

        // Get audio file path  which is clean
        let mut path = env::current_exe().expect("Failed to get executable path");
        path.pop();
        path.push("data");
        path.push(file_name);

        /*let current_audio_title= list_audio_clone.get((current_nb_audios_clone_u32 - 1) as usize).map_or_else(|| String::from("None"), |inner_value| {
            inner_value.audios.get(inner_value.current_audio_index as usize).unwrap().title.to_string()
        });*/
        //tui_print(format!("Start playing : {}", current_audio_title).as_str());

        // Spawn a thread to play the audio
        thread::spawn(move || {
            let mut audio_player = audio_player_clone.lock().unwrap();
            let mut current_audio_time = current_audio_time_clone.lock().unwrap();
            Self::exec_play_audio(&path.to_string_lossy().to_string(), &mut audio_player, &mut current_audio_time);
        });

    }

    fn exec_play_audio(path: &str, audio_player: &Sink, current_audio_time: &mut Duration){
         //println!("{:?}", format!("{}\\{}.mp3", dir_path, file_name));
        // try to open the audio file
        let file = File::open(path).map_or_else(|_| None,|file| Some(file));
        if !(matches!(file, None)){
            let audio_source = rodio::Decoder::new(BufReader::new(file.unwrap())).unwrap();

            *current_audio_time = audio_source.total_duration().unwrap_or(Duration::new(0, 0));
            println!("around {:?} seconds", current_audio_time);
            println!("---------------------------------------");

            // Play the audio file
            audio_player.append(audio_source);
            audio_player.play();
        }
    }
}



#[derive(Clone)]
pub struct PlaylistKa{
    pub title: String,
    pub audios: Vec<AudioKa>,
    pub current_audio_index: u32,
}

impl PlaylistKa  {
    pub fn new(title: String, audios: Vec<AudioKa>) -> Self{
        Self{
            title,
            audios,
            current_audio_index: 0,
        }
    }
    pub fn add(&mut self, ak: AudioKa){
        self.audios.push(ak);
    }
}

// Implement the Display trait for PlaylistKa
impl Display for PlaylistKa
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let formatted_playlist = format!(
            "PlaylistKa {{\n    title: {},\n    audios: {:?},\n    current_audio_index: {}\n}}",
            self.title, self.audios, self.current_audio_index
        );

        write!(f, "{}", formatted_playlist)?;

        Ok(())
    }
}



#[derive(Debug)]
#[derive(Clone)]
pub  struct AudioKa{
    pub title: String,
    pub url: String,
}

impl AudioKa {
    pub fn new(title: String, url: String) -> Self{
        Self{
            title,
            url,
        }
    }
    pub fn new_simple(title: String) -> Self{
        Self{
            title,
            url: String::from(""),
        }
    }
}

// Implement the Display trait for AudioKa
impl Display for AudioKa
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let formatted_playlist = format!(
            "AudioKa {{\n    title: {},\n    url: {:?},\n}}",
            self.title, self.url,
        );

        write!(f, "{}", formatted_playlist)?;

        Ok(())
    }
}



pub struct MediaControlsInternal{
    pub media_controls: MediaControls,
    pub media_controls_status: bool,
}

impl MediaControlsInternal{
    pub fn new(mc: MediaControls) -> Self{
        Self{
            media_controls: mc,
            media_controls_status: true,
        }
    }

    pub fn attach_os_notify(ap_raw: &mut Arc<Mutex<Option<AudioPlayer>>>, mci_raw: &mut Arc<Mutex<Option<MediaControlsInternal>>>) {
        let ap_clone = ap_raw.clone();
        let mut binding_ap = ap_clone.lock().unwrap();
        let mut binding_ap_none = AudioPlayer::new_none();
        let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

        if !(matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Disabled)) {

            let ap_clone_thread = ap.clone();

            if let Some(mci) = mci_raw.lock().unwrap().as_mut() {
                mci.media_controls.attach(move |event: MediaControlEvent| {
                    let mut ap = ap_clone_thread.clone();
                    match event {
                        MediaControlEvent::Pause => {
                            ap.pause();
                        },
                        MediaControlEvent::Play => {
                            ap.resume();
                        },
                        MediaControlEvent::Next =>{
                            ap.next_audio();
                        }
                        _ => {

                        }
                    }
                }).expect("Media Control panic");
            }
        }
    }

}
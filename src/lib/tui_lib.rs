use std::io;
use std::io::Write;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};

#[repr(u32)]
pub enum GeneralSignal {
    Exit = 0,
    InvalidInput = 100,
    Reboot = 200,
    Nothing = 300,
    ValidInput(u32),
}

pub fn tui_print(text: &str){
    io::stdout().write_all(b"\x1B[2J\x1B[1;1H").unwrap();
    let size: u64 = 60;
    let repeated_egale = '='.to_string().repeat(size as usize);
    println!("{}", repeated_egale);
    println!("{}", text);
    println!("{}", repeated_egale);
}

pub fn display_menu(choices: &Vec<String>, choices_quit: &Vec<String>) -> GeneralSignal {
    io::stdout().write_all(b"\x1B[2J\x1B[1;1H").unwrap();
    //################################################################
    //    Vars
    //################################################################
    let mut input = String::new();
    let size: u64 = 60;
    let repeated_egale = '='.to_string().repeat(size as usize);

    //################################################################
    //    Display
    //################################################################
    execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
    println!("{}", repeated_egale);
    println!("{}", repeated_egale);
    println!("  Kasonak");
    println!("{}", repeated_egale);
    for (index, choice) in choices.iter().enumerate(){
        println!("  {} - {}", index+1, choice);
    }
    println!("{}", repeated_egale);
    for (index, choice_quit) in choices_quit.iter().enumerate(){
        println!("  {} - {}", index+1+choices.len(), choice_quit);
    }
    println!("{}", repeated_egale);
    println!("{}", repeated_egale);

    //################################################################
    //    Input
    //################################################################
    io::stdin()
        .read_line(&mut input)
        .expect("ERROR while reading the result");
    let input_nb: u32 = match input.trim().parse() {
        Ok(parsed_number) => parsed_number,
        Err(_) => {
            return GeneralSignal::InvalidInput;
        }
    };
    //################################################################
    //    Quit Action
    //################################################################
    let menu_size = (choices.len() + choices_quit.len()) as u32;
    match input_nb {
        n if n == menu_size => {
            return GeneralSignal::Exit;
        },
        _ => {
            return GeneralSignal::ValidInput(input_nb);
        },
    }
    //################################################################
    //    End
    //################################################################
}

pub fn ask_prompt(title: &str) -> String{
    let mut input_research = String::new();
    println!("{}", title);
    io::stdin()
        .read_line(&mut input_research)
        .expect("ERROR while reading the result");
    return input_research;
}

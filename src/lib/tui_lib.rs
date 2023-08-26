use std::io;
use std::io::Write;
use std::process::exit;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};

pub fn tui_print(text: &str){
    io::stdout().write_all(b"\x1B[2J\x1B[1;1H").unwrap();
    let size: u64 = 60;
    let repeated_egale = '='.to_string().repeat(size as usize);
    println!("{}", repeated_egale);
    println!("{}", text);
    println!("{}", repeated_egale);
}

pub fn display_menu(choices: &Vec<String>, choices_quit: &Vec<String>, quit: bool) -> i32{
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
    let input_nb: i32 = match input.trim().parse() {
        Ok(parsed_number) => parsed_number,
        Err(_) => {
            println!("Invalid input");
            return -2;
        }
    };
    //################################################################
    //    Quit Action
    //################################################################
    let menu_size = (choices.len() + choices_quit.len()) as i32;
    match input_nb {
        n if n == menu_size => {
            if quit{
                exit(0);
            }else {
                return -1;
            }
        },
        _ => {},
    }
    //################################################################
    //    End
    //################################################################
    return input_nb;
}

pub fn ask_prompt(title: &str) -> String{
    let mut input_research = String::new();
    println!("{}", title);
    io::stdin()
        .read_line(&mut input_research)
        .expect("ERROR while reading the result");
    return input_research;
}

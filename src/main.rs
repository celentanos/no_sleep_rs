extern crate getopts;
extern crate kernel32;

use getopts::Options;
use std::collections::BTreeMap;
use std::env;
use std::{thread, time};

const ES_DISPLAY_REQUIRED: u32 = 0x00000002;
const ES_SYSTEM_REQUIRED: u32 = 0x00000001;

const TIMER_MIN: u64 = 1000;
const TIMER_MAX: u64 = 60000;

struct Mode {
    mode: u32,
    name: String,
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "print version");
    opts.optopt(
        "m",
        "mode",
        "set sleep prevention mode,\n\
         MODE = \n\
            1: system\n\
            2: display (default)\n\
            3: system and display",
        "MODE",
    );
    opts.optopt("t", "timer", "set timer (default = 1000ms)", "TIMER");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f.to_string()),
    };

    // help --------------------------------------------------------------------
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    // version -----------------------------------------------------------------
    if matches.opt_present("v") {
        const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
        println!("v{}", VERSION.unwrap_or("unknown"));
        return;
    }

    // timer -------------------------------------------------------------------
    let mut timer: u64;
    match matches.opt_str("t") {
        Some(x) => {
            timer = x.parse::<u64>().unwrap();
            if timer < TIMER_MIN {
                timer = TIMER_MIN;
            }
            if timer > TIMER_MAX {
                timer = TIMER_MAX;
            }
        }
        None => timer = TIMER_MIN,
    }
    println!("timer: {}ms", timer);

    // mode --------------------------------------------------------------------
    let mut mode_map = BTreeMap::new();
    mode_map.insert(
        ES_SYSTEM_REQUIRED,
        Mode {
            mode: ES_SYSTEM_REQUIRED,
            name: String::from("ES_SYSTEM_REQUIRED"),
        },
    );
    mode_map.insert(
        ES_DISPLAY_REQUIRED,
        Mode {
            mode: ES_DISPLAY_REQUIRED,
            name: String::from("ES_DISPLAY_REQUIRED"),
        },
    );

    let mut mode_list: Vec<&Mode> = Vec::new();

    match matches.opt_str("m") {
        Some(x) => match x.as_ref() {
            "1" => mode_list.push(mode_map.get(&ES_SYSTEM_REQUIRED).unwrap()),
            "2" => mode_list.push(mode_map.get(&ES_DISPLAY_REQUIRED).unwrap()),
            "3" => {
                mode_list.push(mode_map.get(&ES_SYSTEM_REQUIRED).unwrap());
                mode_list.push(mode_map.get(&ES_DISPLAY_REQUIRED).unwrap());
            }
            _ => mode_list.push(mode_map.get(&ES_DISPLAY_REQUIRED).unwrap()),
        },
        None => mode_list.push(mode_map.get(&ES_DISPLAY_REQUIRED).unwrap()),
    }
    for mode in mode_list.iter() {
        println!("mode: {}", mode.name);
    }

    // task --------------------------------------------------------------------
    println!(
        "entering sleep prevention mode...\n\
         press 'Ctrl+c' to close this window and exit this program"
    );

    let delay = time::Duration::from_millis(timer);

    loop {
        for _mode in mode_list.iter() {
            #[cfg(target_os = "windows")]
            unsafe {
                kernel32::SetThreadExecutionState(_mode.mode);
            }
        }
        thread::sleep(delay);
    }
}

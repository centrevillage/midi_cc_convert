extern crate portmidi as pm;
extern crate getopts;

use std::time::Duration;
use std::sync::mpsc;
use std::thread;
use std::io;
use std::io::Write;
use std::io::Read;
use getopts::Options;
use std::env;
use std::collections::HashMap;
use std::fs::File;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} CC_MAP_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn select_device<'a>(devices: &'a Vec<pm::DeviceInfo>, device_name: Option<String>) -> pm::DeviceInfo {
    if let Some(valid_name) = device_name {
        if let Some(device) = devices.iter().find(|d| d.name().as_str() == valid_name.as_str()) {
            return device.clone();
        }
    }
    let mut instr = String::new();
    loop {
        println!("[MIDI {} devices]", if devices.first().unwrap().is_input() {"IN"} else {"OUT"});
        for d in devices {
            println!("{}: {}", d.id(), d.name());
        }
        print!("Please select a device number: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut instr).expect("Failed to read line");
        if let Ok(number) = instr.trim().parse::<i32>() {
            if let Some(result) = devices.iter().find(|d| d.id() == number) {
                return result.clone();
            } else {
                println!("Selected device number is invalid.");
            }
        } else {
            println!("Invalid number. {}", instr);
        }
        instr.clear();
    }
}

struct Config {
    debug: bool,
    in_device_name: Option<String>,
    out_device_name: Option<String>,
    mapping: HashMap<u8, u8>
}

impl Config {
    fn new(mapping: HashMap<u8, u8>) -> Config {
        Config {
            debug: false,
            in_device_name: None,
            out_device_name: None,
            mapping: mapping
        }
    }
}

fn parse_mapping(file_path: &str) -> HashMap<u8, u8> {
    let mut mapping = HashMap::new();
    let mut f = File::open(file_path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    for line in s.split("\n") {
        let key_value: Vec<&str> = line.split(',').collect();
        if key_value.len() >= 2 {
            mapping.insert(key_value[0].to_string().trim().parse::<u8>().unwrap(), key_value[1].to_string().trim().parse::<u8>().unwrap());
        }
    }
    mapping
}

fn parse_options() -> Config {
    let mut opts = Options::new();
    opts.optopt("i", "", "MIDI input device name", "NAME");
    opts.optopt("o", "", "MIDI output device name", "NAME");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "print debug infos");
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage("midi_cc_convert", opts);
        std::process::exit(-1);
    }

    let program = args[0].clone();
    let cc_map_file_path = args[1].clone();

    let mapping = parse_mapping(&cc_map_file_path);

    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(_) => { 
            return Config::new(mapping);
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(1);
    }
    Config {debug: matches.opt_present("d"), in_device_name: matches.opt_str("i"), out_device_name: matches.opt_str("o"), mapping: mapping}
}

fn main() {
    let config = parse_options();

    // initialize the PortMidi context.
    let context = pm::PortMidi::new().unwrap();
    const BUF_LEN: usize = 1024;
    let (tx, rx) = mpsc::channel();

    let (in_devices, out_devices): (Vec<pm::DeviceInfo>, Vec<pm::DeviceInfo>) = context.devices()
                                             .unwrap()
                                             .into_iter()
                                             .partition(|dev| dev.is_input());

    if in_devices.is_empty() || out_devices.is_empty() {
        println!("MIDI device is not exist.");
        std::process::exit(-1);
    }
    
    let in_device = select_device(&in_devices, config.in_device_name);
    let in_port = context.input_port(in_device, BUF_LEN).expect("Invalid MIDI port!");
    let out_device = select_device(&out_devices, config.out_device_name);
    let mut out_port = context.output_port(out_device, BUF_LEN).expect("Invalid MIDI port!");

    thread::spawn(move || {
        loop {
            if let Ok(Some(events)) = in_port.read_n(BUF_LEN) {
                tx.send(events).unwrap();
            }
            thread::sleep(Duration::new(0, 100000));
        }
    });

    loop {
        let events = rx.recv().unwrap();
        for event in events {
            let new_event = match event.message.status {
                0xB0 ... 0xBF => pm::MidiEvent {
                                    message: pm::MidiMessage {
                                        status: event.message.status,
                                        data1: (config.mapping.get(&event.message.data1).unwrap_or(&event.message.data1)).clone(),
                                        data2: event.message.data2
                                    },
                                    timestamp: event.timestamp
                                 },
                            _ => event.clone()
            };
            if config.debug {
                if let 0xB0 ... 0xBF = event.message.status {
                    println!("{:?}", new_event);
                }
            }
            out_port.write_event(new_event).unwrap();
        }
    }
}

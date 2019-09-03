use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn load_paths(sys: &str, games: &mut HashMap<String, String>) {
    let paths = fs::read_dir(format!("roms/{}/", sys)).unwrap();
    for path in paths {
        let npath: String = path.unwrap().path().to_str().unwrap().to_owned();
        let ucpath = npath.to_uppercase();
        games.insert(ucpath, npath);
    }
}

fn pick_random(games: &HashMap<String, String>) -> String {
    let idx = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_millis() as usize) % games.len();
    let mut i = 0;
    for (a, _) in games {
        if i == idx {
            if !a.ends_with(&a[5..8]){
                return pick_random(games);
            }
            return a.to_string();
        }
        i += 1;
    }
    String::new()
}

fn main() {
    let wd = std::env::current_exe().unwrap()
        .parent().unwrap()
        .parent().unwrap().to_str()
        .unwrap().to_owned();

    std::env::set_current_dir(&Path::new(&wd)).unwrap();

    let device_tag = "HID".to_owned();

    let mut file = File::open("/proc/bus/input/devices").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    if !device_tag.contains("Handlers=") {
        println!("Card swiper not connected.");
        println!("The only model currently supported is MSR90");
        return;
    }
    let dev = contents.split(&device_tag).collect::<Vec<&str>>()[1]
        .split("Handlers=")
        .collect::<Vec<&str>>()[1]
        .split("\n")
        .collect::<Vec<&str>>()[0]
        .split("event")
        .collect::<Vec<&str>>()[1]
        .split(" ")
        .collect::<Vec<&str>>()[0]
        .to_owned();

    let dev = format!("/dev/input/event{}", dev);

    let mut games = HashMap::new();
    load_paths("nes", &mut games);
    load_paths("smc", &mut games);
    load_paths("sms", &mut games);

    let mut game = pick_random(&games);
    loop {
        let mut child: Option<Child> = None;
        if let Some(res) = games.get(&game) {
            game = res.to_owned();
            println!(">> LOADING {}", game);

            if game.starts_with("roms/nes") {
                match Command::new("bin/nestopia")
                    .args(&["-f".to_string(), game])
                    .env("MESA_GL_VERSION_OVERRIDE", "3.2")
                    .spawn()
                {
                    Ok(res) => child = Some(res),
                    Err(error) => eprintln!("error: {}", error),
                }
            } else if game.starts_with("roms/sms") {
                match Command::new("bin/osmose").args(&["-fs", "-nn2x", "-joy", &game]).spawn() {
                    Ok(res) => child = Some(res),
                    Err(error) => eprintln!("error: {}", error),
                }
            } else if game.starts_with("roms/smc") {
                match Command::new("bin/snes9x").args(&[game]).spawn() {
                    Ok(res) => child = Some(res),
                    Err(error) => eprintln!("error: {}", error),
                }
            } else {
                println!("!! INVALID SYSTEM");
            }
        } else {
            println!("Game not found: {}", game);
        }

        let mut cmd = Command::new("evtest")
            .args(&[dev.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to execute 'evtest' on card swiper");

        let stdout = cmd.stdout.as_mut().expect("failed to get card swiper stdout");
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        let mut codes = Vec::new();
        for line in stdout_lines {
            if let Ok(line) = line {
                if line.contains("(EV_KEY)") && line.contains("(KEY_") && line.contains(", value 0")
                {
                    let code = line.split("(KEY_").collect::<Vec<&str>>()[1]
                        .to_owned()
                        .split("),")
                        .collect::<Vec<&str>>()[0]
                        .to_owned();
                    codes.push(code.clone());
                    if code == "ENTER" {
                        break;
                    }
                }
            }
        }
        let line = parse_codes(codes).split("?").collect::<Vec<&str>>()[0].to_owned();
        game = format!("ROMS/{}", line[1..].to_owned());
        cmd.kill().unwrap_or_default();
        if let Some(child) = &mut child {
            child.kill().unwrap_or_default();
        }
    }
}

fn parse_codes(codes: Vec<String>) -> String {
    let mut out = String::new();
    let mut shifton = false;
    for code in &codes {
        let code: &str = &code[..];
        if code == "RIGHTSHIFT" || code == "LEFTSHIFT" {
            shifton = true;
            continue;
        }
        if code.len() == 1 && code.as_bytes()[0] >= b'A' && code.as_bytes()[0] <= b'Z' {
            if shifton {
                out.push_str(code);
            } else {
                out.push_str(&code.to_lowercase());
            }
        } else if code.len() == 1 && code.as_bytes()[0] >= b'0' && code.as_bytes()[0] <= b'9' {
            if shifton {
                out.push_str(match code {
                    "1" => "!",
                    "2" => "@",
                    "3" => "#",
                    "4" => "$",
                    "5" => "%",
                    "6" => "^",
                    "7" => "&",
                    "8" => "*",
                    "9" => "(",
                    "0" => ")",
                    _ => "?",
                });
            } else {
                out.push_str(code);
            }
        } else {
            out.push_str(match code {
                "GRAVE" => {
                    if !shifton {
                        "~"
                    } else {
                        "`"
                    }
                }
                "MINUS" => {
                    if !shifton {
                        "-"
                    } else {
                        "_"
                    }
                }
                "EQUAL" => {
                    if !shifton {
                        "="
                    } else {
                        "+"
                    }
                }
                "LEFTBRACE" => {
                    if !shifton {
                        "["
                    } else {
                        "{"
                    }
                }
                "RIGHTBRACE" => {
                    if !shifton {
                        "]"
                    } else {
                        "}"
                    }
                }
                "BACKSLASH" => {
                    if !shifton {
                        "\\"
                    } else {
                        "|"
                    }
                }
                "SEMICOLON" => {
                    if !shifton {
                        ";"
                    } else {
                        ":"
                    }
                }
                "APOSTROPHE" => {
                    if !shifton {
                        "'"
                    } else {
                        "\""
                    }
                }
                "COMMA" => {
                    if !shifton {
                        ","
                    } else {
                        "<"
                    }
                }
                "DOT" => {
                    if !shifton {
                        "."
                    } else {
                        ">"
                    }
                }
                "SLASH" => {
                    if !shifton {
                        "/"
                    } else {
                        "?"
                    }
                }
                "SPACE" => " ",
                "ENTER" => "\n",
                _ => "?",
            });
        }
        shifton = false;
    }
    out
}

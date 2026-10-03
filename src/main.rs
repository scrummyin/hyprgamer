use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::Command;

use jqr::run;

const JQ_FILTER_FULLSCREEN_GET_PID: &str = ".[] | select(.fullscreen != 0) | .pid";

fn get_hyprland_socket_path() -> String {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR expected to be set");
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("HYPRLAND_INSTANCE_SIGNATURE expected to be set");
    return format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock");
}

fn add_gamemode_to_pid(pid: String) {
    match Command::new("gamemoded").arg(format!("-s{}", pid)).output() {
        Ok(output) => {
            if String::from_utf8_lossy(&output.stdout).contains("registered") {
                println!("pid({}) already registered", pid);
                return;
            }
        },
        Err(err) => eprintln!("Gamemode errored lookup up pid {} with ({})", pid, err),
    }

    println!("Activating gamemode for pid {}", pid);

    let mut child = Command::new("gamemoded").arg(format!("-r{}", pid)).spawn().expect("failed to attach gamemode, HINT: is it installed");
    child.wait().expect("Gamemode didn't attach correctly");
}

fn get_fullscreened_pids() -> Vec<String> {
    match Command::new("hyprctl").arg("clients").arg("-j").output() {
        Ok(output) => {
            let output_str = std::str::from_utf8(&output.stdout).expect("hyprctl output is not valid UTF-8");
            let results = run(JQ_FILTER_FULLSCREEN_GET_PID, output_str).expect("Failed to parse json output");
            return results.into_iter().map(|x| x.to_string()).collect();
        }
        Err(err) => {
            eprintln!("hyprctl lookup failed {}", err)
        }
    }
    Vec::new()
}

fn check_for_fullscreen_apps_and_add_gamemode() {
    for pid in get_fullscreened_pids() {
        println!("Found game process with pid {}", pid);
        add_gamemode_to_pid(pid.to_string());
    }
}

fn trigger_gamemode_on_fullscreen_toggles(stream: UnixStream) {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap() {
            s if s.contains("fullscreen>>") => {
                println!("A window has toggled its fullscreen state");
                check_for_fullscreen_apps_and_add_gamemode();
            },
            _ => (),
        }
    }
}

fn main() -> std::io::Result<()> {
    let socket_path = get_hyprland_socket_path();

    let _socket = match UnixStream::connect(socket_path) {
        Ok(stream) => {
            println!("Listening to hyprland socket");
            trigger_gamemode_on_fullscreen_toggles(stream)
        }
        Err(err) => {
            return Err(err);
        }
    };
    Ok(())
}
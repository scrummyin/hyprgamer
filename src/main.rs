use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::Command;

use sysinfo::{System, Pid, Process};

fn get_hyprland_socket_path() -> String {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR expected to be set");
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("HYPRLAND_INSTANCE_SIGNATURE expected to be set");
    return format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock");
}

fn proc_has_string_in_args(proc: &Process) -> bool {
    match proc.cmd() {
        [_one, steam_launch_command, app_id, ..] => {
            return steam_launch_command.to_string_lossy().contains("SteamLaunch") &&
                app_id.to_string_lossy().contains("AppId=")
        },
        _ => return false,
    }
}

fn find_pids_for_steam_apps() -> Vec<Pid> {
    let sys = System::new_all();
    let mut matched_pids = Vec::new();

    for (_, process) in sys.processes() {
        if proc_has_string_in_args(process) {
            matched_pids.push(process.pid());
        }
    }

    return matched_pids;
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

fn check_for_steam_apps_and_add_gamemode() {
    for pid in find_pids_for_steam_apps() {
        println!("Found game process with pid {}", pid);
        add_gamemode_to_pid(pid.to_string());
    }
}

fn filter_steam_apps_and_attach_gamemode(stream: UnixStream) {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap() {
            s if s.contains("fullscreen>>") => {
                println!("A window has toggled its fullscreen state");
                check_for_steam_apps_and_add_gamemode();
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
            filter_steam_apps_and_attach_gamemode(stream)
        }
        Err(err) => {
            return Err(err);
        }
    };
    Ok(())
}
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

fn find_steam_app_id(socket_message: String) -> Option<String> {
    let mut msg_parts = socket_message.split(',');
    match (msg_parts.next(), msg_parts.next(), msg_parts.next()) {
        (Some(_), Some(_), Some(steam_app_id_with_prefix), ..) => {
            let mut app_id_parts = steam_app_id_with_prefix.split('_');
            match (app_id_parts.next(), app_id_parts.next(), app_id_parts.next()) {
                (Some("steam"), Some("app"), Some(steam_app_id), ..) => {
                    return Some(steam_app_id.to_owned());
                },
                _ => println!("Failed to find steam app id"),
            }
        }
        _ => println!("Failed to find steam app id"),
    }
    return None
}

fn proc_has_string_in_args(proc: &Process, matcher: String) -> bool {
    match proc.cmd() {
        [_one, steam_launch_command, app_id, ..] => {
            return steam_launch_command.to_string_lossy().contains("SteamLaunch") &&
                app_id.to_string_lossy().contains(&matcher)
        },
        _ => return false,
    }
}

fn find_pid_for_steam_app_id(steam_app_id: String) -> Option<Pid> {
    let sys = System::new_all();
    let steam_launch_string = format!("AppId={}", steam_app_id);
    for (_, process) in sys.processes() {
        if proc_has_string_in_args(process, steam_launch_string.clone()) {
            return Some(process.pid());
        }
    }
    return None;
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

fn check_if_steam_app_and_add_gamemode(socket_message: String) {
    match find_steam_app_id(socket_message.clone()) {
        Some(steam_app_id) => {
            match find_pid_for_steam_app_id(steam_app_id.clone()) {
                Some(pid) => {
                    println!("Found pid {} to register for steam app {}", pid, steam_app_id);
                    add_gamemode_to_pid(pid.to_string());
                },
                _ => println!("Failed for {}", socket_message.clone()),
            }
        },
        None => {println!("statement {} didn't result in match", socket_message)},
    }
}

fn filter_steam_apps_and_attach_gamemode(stream: UnixStream) {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap() {
            s if s.contains("openwindow>>") && s.contains("steam_app_") => {
                println!("checking open window {}", s);
                check_if_steam_app_and_add_gamemode(s);
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

# hyprgamer

Automatically enables [GameMode](https://github.com/FeralInteractive/gamemode) when a window goes fullscreen within [Hyprland](https://github.com/hyprwm/Hyprland).

No more modifying every game launcher script to include `gamemoderun`.

## How to use

Install hyprgamer and execute it inside a hyprland session.

To do so automatically, add the following exec_cmd line to your hyprland.start function in your **Hyprland config** (`~/.config/hypr/hyprland.lua`):
```lua
hl.on("hyprland.start", function()
...
    hl.exec_cmd("hyprgamer")
...
end)
```

## Requirements

- **Hyprland** 0.40+ (running, with `hyprctl` in PATH)
- **gamemoded** (from your distro's package manager or [the Feral repo](https://github.com/FeralInteractive/gamemode))

A running hyprland session should provide the required environment variables of `XDG_RUNTIME_DIR` and `HYPRLAND_INSTANCE_SIGNATURE`.

## Oh no hyprgamer activated gamemode on my fullscreen calculator

You can always manually remove gamemode from a process by running the below (NOTE: no space between the `-r` and the pid):

```bash
gamemoded -r<pid>
```

That also happens automatically when you close the calculator.

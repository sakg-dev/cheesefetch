use sysinfo::{
    System,
    Motherboard,
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::{ Command, Stdio };
use std::io::Write;
use std::io;
use std::str;
use std::fs;
use regex::Regex;
use colored::Colorize;
use terminal_size::{Width, Height, terminal_size};

struct Cpu {
    brand: String,
    mul: u32,
    frequency: f32
}

struct Resolution {
    width: u32,
    height: u32,
    refresh_rate: f32
}

struct Info {
    name: Option<String>,
    value: String
}

const GENERAL_TAG_REG:&str = r"[a-z:-]+";

// TODO: many unused vars and functions (search UNUSED to see)

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    fn get_os() -> String {
        let name = System::name().unwrap();
        let os_version = System::os_version().unwrap();
        let cpu_arch = System::cpu_arch();
        format!("{} {} {}", name, os_version, cpu_arch)
    }
    fn get_host() -> String {
        System::host_name().unwrap()
    }
    fn get_kernel() -> String {
        System::kernel_long_version()
    }
    fn get_uptime() -> String{
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let boot_time = System::boot_time();
        sec_to_readeable_time(current_time - boot_time)
    }
    fn get_mem(sys:&System) -> String {
        let available_mem = sys.total_memory() / (1024*1024) as u64;
        let free_mem = sys.free_memory() / (1024*1024) as u64;
        format!("{}MB / {}MB", free_mem, available_mem)
    }
    fn _get_board() -> String { // UNUSED
        let mboard = Motherboard::new().unwrap();
        let board_name = mboard.name().unwrap();
        let board_vendor = mboard.vendor_name().unwrap();
        format!("{} {}", board_vendor, board_name)
    }
    fn get_cpus(sys:&System) -> String {
        let mut cpus: Vec<Cpu> = Vec::new();
        for cpu in sys.cpus() {
            if cpus.iter().any(|c| c.brand.as_str() == cpu.brand()) {
                let idx = cpus.iter().position(|c| c.brand.as_str() == cpu.brand()).unwrap();
                cpus[idx].mul += 1;
                let current_freq = cpu.frequency() as f32 / 1000.0;
                if cpus[idx].frequency < current_freq {
                    cpus[idx].frequency = current_freq;
                }
            } else {
                let new_cpu = Cpu{
                    brand: cpu.brand().to_string(),
                    mul: 1,
                    frequency: cpu.frequency() as f32 / 1000.0
                };
                cpus.push(new_cpu);
            }
        }

        let mut cpus_str = String::new();

        for cpu in cpus {
            cpus_str.push_str(format!("{} ({}) @ {:.1}GHz", cpu.brand, cpu.mul, cpu.frequency).as_str());
        }
        cpus_str
    }

    fn get_resolution() -> String{
        let resolution = if cfg!(target_os = "linux") {
            let cmd = Command::new("xrandr")
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let grep = Command::new("grep")
                .arg("*")
                .stdin(Stdio::from(cmd.stdout.unwrap()))
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let output = grep.wait_with_output().unwrap();
            let result = str::from_utf8(&output.stdout).unwrap();
            let outputs =  result.trim().split(" ").collect::<Vec<_>>().iter().filter(|&s| s != &"").cloned().collect::<Vec<_>>();
            let display_size:Vec<u32> = outputs[0].split("x").collect::<Vec<_>>().iter().map(|&s| s.parse::<u32>().unwrap()).collect();
            let refresh_rate = outputs[1].split("*").collect::<Vec<_>>()[0].parse::<f32>().unwrap();
            Resolution {
                width: display_size[0],
                height: display_size[1],
                refresh_rate: refresh_rate
            }
        } else {
            // FUTURE-TODO: Write Command for Windows and MacOs, for now returning dummy result
            // for windows this cmd works(used ai to get this cmd): Get-CimInstance Win32_VideoController | ForEach-Object { "$($_.VideoModeDescription), $($_.CurrentRefreshRate)" }
            Resolution {
                width: 1600,
                height: 900,
                refresh_rate: 120.0 
            }
        };

        format!("{}x{} {}Hz", resolution.width, resolution.height, resolution.refresh_rate)
    }
    fn get_wm() -> String {
        let mut bind = Command::new("bash");
        let wm_binary = bind.args(["-c", r###"id=$(xprop -root -notype _NET_SUPPORTING_WM_CHECK) && id=${id##* } && wm=$(xprop -id "$id" -notype -len 100 -f _NET_WM_NAME 8t) && wm=${wm/*WM_NAME = } && wm=${wm/\"} && wm=${wm/\"*} && printf $wm"###]).output().unwrap().stdout;
        str::from_utf8(&wm_binary).unwrap().to_string() 
    }
    fn get_shell() -> String {
        let system = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::everything().with_processes(
                sysinfo::ProcessRefreshKind::everything()
            )
        );
        let my_pid = sysinfo::get_current_pid().unwrap();
        let parent_pid = system.process(my_pid).unwrap().parent().unwrap();
        let parent_process = system.process(parent_pid).unwrap();
        parent_process.name().to_str().unwrap().to_string()
    }

    // ---INFOs---
    let host = get_host();
    let _os = get_os(); // UNUSED
    let kernel = get_kernel();
    let uptime = get_uptime();
    // TODO: IDK how to get packages, ig i will have to identify the package manager and do manually??
    let shell = get_shell();
    let resolution = get_resolution();
    let wm = get_wm();
    let cpus = get_cpus(&sys);
    let mem = get_mem(&sys);

    let infos: Vec<Info> = vec![
        Info{ name: None, value: format!("<cyan>{}</cyan>", host.clone()) },
        Info{ name: None, value: "-".repeat(host.len()) },
        Info{ name: Some("kernel".to_string()), value: kernel },
        Info{ name: Some("uptime".to_string()), value: uptime },
        Info{ name: Some("packages".to_string()), value: "undefined".to_string() },
        Info{ name: Some("shell".to_string()), value: shell },
        Info{ name: Some("resolution".to_string()), value: resolution },
        Info{ name: Some("wm".to_string()), value: wm },
        Info{ name: Some("cpus".to_string()), value: cpus },
        Info{ name: Some("gpu".to_string()), value: "undefined".to_string() },
        Info{ name: Some("memory".to_string()), value: mem },
        Info{ name: None, value: "".to_string() },
        Info{ name: None, value: "<bg:black>   </bg:black><bg:red>   </bg:red><bg:green>   </bg:green><bg:yellow>   </bg:yellow><bg:blue>   </bg:blue><bg:magenta>   </bg:magenta><bg:cyan>   </bg:cyan><bg:bright-black>   </bg:bright-black>".to_string() },
        Info{ name: None, value: "<bg:brighter-black>   </bg:brighter-black><bg:bright-red>   </bg:bright-red><bg:bright-green>   </bg:bright-green><bg:bright-yellow>   </bg:bright-yellow><bg:bright-blue>   </bg:bright-blue><bg:bright-magenta>   </bg:bright-magenta><bg:bright-cyan>   </bg:bright-cyan><bg:brightest-black>   </bg:brightest-black>".to_string() }
    ];
   
    display_everything(infos)
}

fn display_everything(infos:Vec<Info>) {
    let ascii = fs::read_to_string("./assets/ascii_arts/simple_cheese.txt").unwrap();
    let ascii_str = ascii.as_str();
    let (ascii_w, ascii_h) = get_ascii_size(ascii_str);
    let (infos_w, infos_h) = get_infos_size(&infos);

    const GAP:u32 = 8;

    let tsize = terminal_size();
    
    if let Some((Width(w), Height(_))) = tsize {
        if (ascii_w + GAP + infos_w) > w.into() {
            // vertical
        } else { // horizontal
            if infos_h > ascii_h {
            } else { // Infos is <+ ascii so run loop in ascii
                let diff = ascii_h - infos_h;
                let infos_start = if diff % 2 == 1 {(diff+1)/2} else {diff/2};

                for (idx, ascii_line) in ascii_str.split("\n").enumerate() {
                    let idx = idx as u32;
                    let mut line_str = String::from(ascii_line); // ascii + gap + info
                    if idx+1 >= infos_start {
                        let info_idx = ((idx+1) - infos_start) as usize;
                        if (info_idx as u32) < infos_h { // if we have info
                            let info = &infos[info_idx];
                            let ascii_space = ascii_w - get_ascii_size(ascii_line).0 + GAP;
                            line_str.push_str(" ".repeat(ascii_space.try_into().unwrap()).as_str());
                            if let Some(name) = &info.name {
                                line_str.push_str(format!("<cyan>{}</cyan>: {}", name , info.value).as_str());
                            } else {
                                line_str.push_str(format!("{}", info.value).as_str());
                            }
                        }
                    }
                    cprintln(line_str);
                }
            }
        }
    }
}

fn print_tagged_text(tag_name: String, text: String) {
    let tagged_str = match tag_name.as_str() {
        "red" => text.red(),
        "green" => text.green(),
        "blue" => text.blue(),
        "yellow" => text.yellow(),
        "cyan" => text.cyan(),
        "orange" => text.truecolor(255, 165, 0),

        "bg:black" => text.on_black(),
        "bg:red" => text.on_red(),
        "bg:green" => text.on_green(),
        "bg:yellow" => text.on_yellow(),
        "bg:blue" => text.on_blue(),
        "bg:magenta" => text.on_magenta(),
        "bg:cyan" => text.on_cyan(),
        "bg:bright-black" => text.on_bright_black(),
        "bg:brighter-black" => text.on_truecolor(79, 79, 79),
        "bg:bright-red" => text.on_bright_red(),
        "bg:bright-green" => text.on_bright_green(),
        "bg:bright-yellow" => text.on_bright_yellow(),
        "bg:bright-blue" => text.on_bright_blue(),
        "bg:bright-magenta" => text.on_bright_magenta(),
        "bg:bright-cyan" => text.on_truecolor(122, 255, 255),
        "bg:brightest-black" => text.on_truecolor(211, 211, 211),
        _ => text.into()
    };
    print!("{}", tagged_str);
}

fn cprintln(txt: String) {
    struct Tag {
        tag_name: String,
        text: String,
    }
    enum Chunk {
        Tagged(Tag),
        Untagged(String)
    }

    let general_text_reg:&str = r"[a-zA-Z0-9!~_+\-|/\\.() :]+";

    let reg = Regex::new(format!(r"(<{GENERAL_TAG_REG}>{general_text_reg}</{GENERAL_TAG_REG}>)|{general_text_reg}").as_str()).unwrap(); // for dividing into chunks
    let tags_reg = Regex::new(format!(r"<{GENERAL_TAG_REG}>{general_text_reg}</{GENERAL_TAG_REG}>").as_str()).unwrap(); // for identifying whether we have tags or not
    let tags_part_reg = Regex::new(format!(r"<{GENERAL_TAG_REG}>|{general_text_reg}").as_str()).unwrap();
 
    let chunks:Vec<Chunk> = reg.find_iter(&txt).map(|m| m.as_str()).map(|m| { // get all chunks regardless tag or untag
        let contains_tag = tags_reg.find(m); // inside each chunk check if it contains tag or not
        if let Some(tag) = contains_tag { // contain tag
            // get color name and other info and add in struct
            let mut infos = tags_part_reg.find_iter(tag.as_str());
            let tag_name = infos.next().unwrap().as_str().replace(&['<', '>'], "");
            let text = infos.next().unwrap().as_str().to_string();
            Chunk::Tagged(Tag{
                tag_name: tag_name,
                text: text
            })
        } else { // if its a text only
            Chunk::Untagged(m.to_string())
        }
    }).collect();

    for chunk in chunks {
        match chunk {
            Chunk::Untagged(text) => print!("{}", text),
            Chunk::Tagged(Tag{tag_name, text}) => print_tagged_text(tag_name, text)
        }
        io::stdout().flush().unwrap();
    };
    println!("");
}

fn get_ascii_size(ascii:&str) -> (u32, u32){
    // return height and for width, returns of largest
    let ascii_iter = ascii.split("\n");
    let height:u32 = ascii_iter.clone().count() as u32;
    let re = Regex::new(format!(r"</*{GENERAL_TAG_REG}>").as_str()).unwrap();
    let mut width:u32 = 0;
    for line in ascii_iter{
        let len = re.replace_all(line, "").len();
        if width < len.try_into().unwrap() {
            width = len as u32;
        }
    }
    (width, height)
}

fn get_infos_size(infos: &Vec<Info>) -> (u32, u32){
    let height:u32 = infos.len() as u32;
    let mut width:u32 = 0;
    for info in infos {
        if let Some(name) = &info.name {
            let mut full_string = String::new();
            full_string.push_str(name);
            full_string.push_str(": ");
            full_string.push_str(&info.value);
            let full_string_len = full_string.len() as u32;
            if full_string_len > width {
                width = full_string_len;
            }
        } else {
            let (w, _) = get_ascii_size(&info.value);
            if w > width {
                width = w
            }
        }
    }
    (width, height)
}

fn sec_to_readeable_time(secs: u64) -> String {
    if secs < 60 {
        format!("{secs} seconds")
    } else if secs < 60 * 60 { // under hr
        let mins = secs/60 as u64;
        let remaining_secs = secs % 60;
        format!("{mins} minutes, {remaining_secs} seconds")
    } else if secs < 60 * 60 * 24 {
        let hrs = secs/3600 as u64;
        let remaining_time = secs % 3600;
        let remaining_mins = remaining_time/60 as u64;
        let remaining_secs = remaining_time % 60;
        format!("{hrs} hours, {remaining_mins} minutes, {remaining_secs} seconds")
    } else {
        let days = secs/(3600*24) as u64;
        let remaining_time = secs % 3600*24;
        let remaining_hrs = remaining_time/3600 as u64;
        let remaining_time = secs % 3600;
        let remaining_mins = remaining_time/60 as u64;
        let remaining_secs = remaining_time % 60; 
        format!("{days} days, {remaining_hrs} hours, {remaining_mins} minutes, {remaining_secs} seconds")
    }
}

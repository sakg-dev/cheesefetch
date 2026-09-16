use sysinfo::{
    System,
    Motherboard,
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::{ Command, Stdio };
use std::str;
use std::io::Write;
use regex::Regex;
use color_print::{ cprintln, cprint };
use std::fs;
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

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    // ---INFOs---
    let os = format!("{} {} {}", System::name().unwrap(), System::os_version().unwrap(), System::cpu_arch());
    let host = System::host_name().unwrap();
    let kernel = System::kernel_long_version();

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let boot_time = System::boot_time();
    let uptime = sec_to_readeable_time(current_time - boot_time);

    let available_mem = sys.total_memory() / (1024*1024) as u64;
    let free_mem = sys.free_memory() / (1024*1024) as u64;

    let mboard = Motherboard::new().unwrap();
    let board_name = mboard.name().unwrap();
    let board_vendor = mboard.vendor_name().unwrap();
    let _board = format!("{} {}", board_vendor, board_name);

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

    // TODO: IDK how to get packages, ig i will have to identify the package manager and do manually??

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
        // TODO: Write Command for Windows and MacOs, for now returning dummy result
        // for windows this cmd works(used ai to get this cmd): Get-CimInstance Win32_VideoController | ForEach-Object { "$($_.VideoModeDescription), $($_.CurrentRefreshRate)" }
        Resolution {
            width: 1600,
            height: 900,
            refresh_rate: 120.0 // high to differentiate from og val
        }
    };
   
    // WM
    let mut bind = Command::new("bash");
    let wm_binary = bind.args(["-c", r###"id=$(xprop -root -notype _NET_SUPPORTING_WM_CHECK) && id=${id##* } && wm=$(xprop -id "$id" -notype -len 100 -f _NET_WM_NAME 8t) && wm=${wm/*WM_NAME = } && wm=${wm/\"} && wm=${wm/\"*} && printf $wm"###]).output().unwrap().stdout;
    let wm = str::from_utf8(&wm_binary).unwrap();
    
    // Shell
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::everything().with_processes(
            sysinfo::ProcessRefreshKind::everything()
        )
    );
    let my_pid = sysinfo::get_current_pid().unwrap();
    let parent_pid = system.process(my_pid).unwrap().parent().unwrap();
    let parent_process = system.process(parent_pid).unwrap();
    let shell = parent_process.name().to_str().unwrap();


    display_everything(host, os, kernel, uptime, "undefined", shell, resolution, wm, cpus, "undefined", free_mem, available_mem)
}

fn display_everything(host:String, os:String, kernel:String, uptime:String, packages:&str, shell:&str, resolution:Resolution, wm:&str, cpus:Vec<Cpu>, gpu:&str, free_mem:u64, available_mem:u64) {
    // idea: get the width and height of the current terminal, according to that plot, either in
    // column or row, keep infos and clr boxes one side, asci on other..

    let tsize = terminal_size();
    println!("{:?}", tsize);

    // System Infos
    cprintln!("<bold, cyan>{}</>", host);
    cprintln!("{}", "—".repeat(host.len()));
    cprintln!("<bold><cyan>OS</>:</> {}", os);
    cprintln!("<bold><cyan>Kernel</>:</> {}", kernel);
    cprintln!("<bold><cyan>Uptime</>:</> {}", uptime);
    cprintln!("<bold><cyan>Packages</>:</> {}", packages);
    cprintln!("<bold><cyan>Shell</>:</> {}", shell);
    cprintln!("<bold><cyan>Resolution</>:</> {}x{} {}Hz", resolution.width, resolution.height, resolution.refresh_rate);
    cprintln!("<bold><cyan>WM</>:</> {}", wm);
    cpu_print(cpus);
    cprintln!("<bold><cyan>GPU</>:</> {}", gpu);
    cprintln!("<bold><cyan>Memory</>:</> {}MB / {}MB", free_mem, available_mem);

    block_clr_print();

    draw_ascii()
}

fn print_tagged_text(tag_name: String, text: String) { // prints using print macro, need to flush outside
    let tagged_str = match tag_name.as_str() {
        "red" => text.red(),
        "green" => text.green(),
        "blue" => text.blue(),
        "yellow" => text.yellow(),
        "orange" => text.truecolor(255, 165, 0),
        _ => text.into()
    };
    print!("{}", tagged_str);
}

fn print_ascii_line(txt: String) {
    #[derive(Debug)]
    struct Tag {
        tag_name: String,
        text: String,
    }
    #[derive(Debug)]
    enum Chunk {
        Tagged(Tag),
        Untagged(String)
    }

    let general_text_reg = r"[a-zA-Z0-9!~_+\-|/\\.() ]+";
    let reg = Regex::new(format!(r"(<[a-z]+>{general_text_reg}</[a-z]+>)|{general_text_reg}").as_str()).unwrap(); // for dividing into chunks
    let tags_reg = Regex::new(format!(r"<[a-z]+>{general_text_reg}</[a-z]+>").as_str()).unwrap(); // for identifying whether we have tags or not
    let tags_part_reg = Regex::new(format!(r"<[a-z]+>|{general_text_reg}").as_str()).unwrap();
 
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
    };
    println!("");
}

fn draw_ascii() {
    let ascii = fs::read_to_string("./assets/ascii_arts/simple_cheese.txt").unwrap();
    for line in ascii.split("\n"){
        print_ascii_line(line.to_string())
    }
}

fn block_clr_print() {
    println!("");

    cprint!("<bg:black>   </>");
    cprint!("<bg:red>   </>");
    cprint!("<bg:green>   </>");
    cprint!("<bg:yellow>   </>");
    cprint!("<bg:blue>   </>");
    cprint!("<bg:magenta>   </>");
    cprint!("<bg:cyan>   </>");
    std::io::stdout().flush().unwrap();

    cprintln!("<bg:bright-black>   </>");

    cprint!("<bg:rgb(79,79,79)>   </>");
    cprint!("<bg:bright-red>   </>");
    cprint!("<bg:bright-green>   </>");
    cprint!("<bg:bright-yellow>   </>");
    cprint!("<bg:bright-blue>   </>");
    cprint!("<bg:bright-magenta>   </>");
    cprint!("<bg:rgb(122,255,255)>   </>");
    std::io::stdout().flush().unwrap();

    cprintln!("<bg:rgb(211,211,211)>   </>");
}

fn cpu_print(cpus: Vec<Cpu>) {
    cprint!("<bold><cyan>CPU</>: </>");
    for cpu in cpus {
        cprint!("{} ({}) @ {:.1}GHz", cpu.brand, cpu.mul, cpu.frequency)
    }
    std::io::stdout().flush().unwrap();
    cprintln!("");
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

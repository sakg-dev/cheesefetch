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

#[derive(Debug)]
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

#[derive(Debug)]
struct Info {
    name: Option<String>,
    value: String
}

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
    fn _get_board() -> String {
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
            // TODO: Write Command for Windows and MacOs, for now returning dummy result
            // for windows this cmd works(used ai to get this cmd): Get-CimInstance Win32_VideoController | ForEach-Object { "$($_.VideoModeDescription), $($_.CurrentRefreshRate)" }
            Resolution {
                width: 1600,
                height: 900,
                refresh_rate: 120.0 // high to differentiate from og val
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
    let _os = get_os(); //TODO: unused
    let kernel = get_kernel();
    let uptime = get_uptime();
    // TODO: IDK how to get packages, ig i will have to identify the package manager and do manually??
    let shell = get_shell();
    let resolution = get_resolution();
    let wm = get_wm();
    let cpus = get_cpus(&sys);
    let mem = get_mem(&sys);

    let infos: Vec<Info> = vec![
        Info{ name: None, value: host.clone() },
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
    ];
   
    display_everything(infos)
}

fn display_everything(infos:Vec<Info>) {
    // idea: get the width and height of the current terminal, according to that plot, either in
    // column or row, keep infos and clr boxes one side, asci on other..
    let ascii = fs::read_to_string("./assets/ascii_arts/simple_cheese.txt").unwrap();
    let ascii_str = ascii.as_str();
    let (ascii_w, _ascii_h) = get_ascii_size(ascii_str);
    const GAP:u32 = 8;

    let tsize = terminal_size();
    let mut infos_lines_taken:u32 = 0;
    
    if let Some((Width(w), Height(_h))) = tsize {
        // println!("{}:{}", w, h);
        if ascii_w*2 > w.into() { // if ascii is more then half of terminal -- vertical
        } else { // -- horizontal
            for _ in infos.iter() {
                infos_lines_taken += 1;
            }


            block_clr_print(&mut infos_lines_taken); 


            let ascii_str_splitted = ascii_str.split("\n");
            let ascii_str_lines = ascii_str_splitted.clone().count() as u32;
            
            if infos_lines_taken > ascii_str_lines { // TODO: wht if they are equal
            } else { // infos lines are less than ascii str lines
                let diff = ascii_str_lines - infos_lines_taken;
                let infos_start = if(diff % 2 == 1){(diff+1)/2} else {diff/2};
                // Now start the infos from infos_start, with the gap as GAP.

                for (idx, line) in ascii_str_splitted.enumerate() {
                    let idx = idx as u32;
                    let mut line_str = String::from(line);
                    if idx+1 >= infos_start {
                        let info_idx = ((idx+1) - infos_start) as usize;
                        if info_idx < infos.len() {
                            let info = &infos[info_idx];
                            let ascii_space = ascii_w - get_ascii_size(line).0 + GAP;
                            line_str.push_str(" ".repeat(ascii_space.try_into().unwrap()).as_str());
                            if let Some(name) = &info.name {
                                line_str.push_str(format!("<cyan>{}</cyan>: {}", name , info.value).as_str());
                            } else {
                                line_str.push_str(format!("<cyan>{}</cyan>", info.value).as_str());
                            }
                        }
                    }
                    print_ascii_line(line_str);
                }
            }
        }
    }
}

fn print_tagged_text(tag_name: String, text: String) { // prints using print macro, need to flush outside
    let tagged_str = match tag_name.as_str() {
        "red" => text.red(),
        "green" => text.green(),
        "blue" => text.blue(),
        "yellow" => text.yellow(),
        "cyan" => text.cyan(),
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

    let general_text_reg = r"[a-zA-Z0-9!~_+\-|/\\.() :]+";
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

fn get_ascii_size(ascii:&str) -> (u32, u32){
    // return height and for width, returns of largest
    let ascii_iter = ascii.split("\n");
    let height:u32 = ascii_iter.clone().count() as u32;
    let re = Regex::new(r"</*[a-z]*>").unwrap();
    let mut width:u32 = 0;
    for line in ascii_iter{
        let len = re.replace_all(line, "").len();
        if width < len.try_into().unwrap() {
            width = len as u32;
        }
    }
    (width, height)
}

fn block_clr_print(infos_lines_taken: &mut u32) {
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
    *infos_lines_taken += 3;
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

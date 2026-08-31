// IDFK how to do things efficiently in an efficient lang

use sysinfo::{
    System,
    Motherboard,
};
use std::time::{SystemTime, UNIX_EPOCH};
// use display_info::DisplayInfo;
// use winit::{
//    event_loop::{EventLoop},
//    window::Window,
// };
use std::process::{ Command, Stdio };
use std::str;
// use regex::Regex; // TODO: use scary regex pls instead of split and other tricks
use color_print::{ cprintln, cprint };

#[derive(Debug)] 
struct Cpu {
    brand: String,
    mul: u32,
    frequency: f32
}

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

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
            // println!("Found duplicate");
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

    // let mut event_loop = EventLoop::new();
    // let window = Window::new(&event_loop).unwrap();
    // window.current_monitor();
    // nothing working perfectly hence running cmd to get as done in freshfetch
    #[derive(Debug)]
    struct Resolution {
        width: u32,
        height: u32,
        refresh_rate: f32
    }
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
        // println!("{:?}", result.trim().split(" ").collect::<Vec<_>>())
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

    // -------------------

    // cprintln!("Hii <c>Hii</>");
    // cprintln!("<bold><bold> A <bold,blue> B </> C </></>");
    cprintln!("<bold, cyan>{}</>", host);
    cprintln!("{}", "—".repeat(host.len()));
    cprintln!("<bold><cyan>OS</>:</> {}", os);
    cprintln!("<bold><cyan>Kernel</>:</> {}", kernel);
    cprintln!("<bold><cyan>Uptime</>:</> {}", uptime);
    cprintln!("<bold><cyan>Packages</>:</> undefined");
    cprintln!("<bold><cyan>Shell</>:</> {}", shell);
    cprintln!("<bold><cyan>Resolution</>:</> {}x{} {}Hz", resolution.width, resolution.height, resolution.refresh_rate);
    cprintln!("<bold><cyan>WM</>:</> {}", wm);
    cpu_print(cpus);
    cprintln!("<bold><cyan>GPU</>:</> {}", "undefined");
    cprintln!("<bold><cyan>Memory</>:</> {}MB / {}MB", free_mem, available_mem);
    block_clr_print();
}

fn block_clr_print() {
    // TODO: flugh thingy
    println!("");
    cprint!("<bg:black>   </>");
    cprint!("<bg:red>   </>");
    cprint!("<bg:green>   </>");
    cprint!("<bg:yellow>   </>");
    cprint!("<bg:blue>   </>");
    cprint!("<bg:magenta>   </>");
    cprint!("<bg:cyan>   </>");
    cprintln!("<bg:bright-black>   </>");
    cprint!("<bg:rgb(79,79,79)>   </>");
    cprint!("<bg:bright-red>   </>");
    cprint!("<bg:bright-green>   </>");
    cprint!("<bg:bright-yellow>   </>");
    cprint!("<bg:bright-blue>   </>");
    cprint!("<bg:bright-magenta>   </>");
    cprint!("<bg:rgb(122,255,255)>   </>");
    cprintln!("<bg:rgb(211,211,211)>   </>");
}

fn cpu_print(cpus: Vec<Cpu>) {
    cprint!("<bold><cyan>CPU</>: </>");
    for cpu in cpus {
        cprint!("{} ({}) @ {:.1}GHz", cpu.brand, cpu.mul, cpu.frequency)
    }
    cprintln!("");
    // TODO: DO the flush thingy
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

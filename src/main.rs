// IDFK how to do things efficiently in an efficient lang

use sysinfo::{
    System,
    Motherboard,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let _os = String::from("{System::name().unwrap()} {System::os_version().unwrap()} {System::cpu_arch()}");
    let _host = System::host_name();
    let _kernel = System::kernel_long_version();

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let boot_time = System::boot_time();
    let _uptime = sec_to_readeable_time(current_time - boot_time);

    let _available_mem = sys.total_memory() / (1024*1024) as u64;
    let _free_mem = sys.free_memory() / (1024*1024) as u64;

    let mboard = Motherboard::new().unwrap();
    let board_name = mboard.name().unwrap();
    let board_vendor = mboard.vendor_name().unwrap();
    let _board = format!("{} {}", board_vendor, board_name);

    #[derive(Debug)]
    struct Cpu {
        brand: String,
        mul: u32,
        frequency: u64
    }
    let mut cpus: Vec<Cpu> = Vec::new();
    for cpu in sys.cpus() {
        if cpus.iter().any(|c| c.brand.as_str() == cpu.brand()) {
            // println!("Found duplicate");
            let idx = cpus.iter().position(|c| c.brand.as_str() == cpu.brand()).unwrap();
            cpus[idx].mul += 1;
            let current_freq = cpu.frequency();
            if cpus[idx].frequency < current_freq {
                cpus[idx].frequency = current_freq;
            }
        } else {
            let new_cpu = Cpu{
                brand: cpu.brand().to_string(),
                mul: 1,
                frequency: cpu.frequency()
            };
            cpus.push(new_cpu);
        }
    }

    // TODO: IDK how to get packages, ig i will have to identify the package manager and do manually??
    

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

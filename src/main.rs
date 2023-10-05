use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};

use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_path = args.get(1).expect("No file path provided");
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut curve = Vec::new();
    for l in reader.lines() {
        let line = l?;
        let mut parts = line.split_whitespace().map(|s| s.parse::<u32>());
        if let (Some(Ok(a)), Some(Ok(b))) = (parts.next(), parts.next()) {
            curve.push((a, b));
        }
    }

    assert!(curve.len() > 1);
    let is_valid_curve = curve.windows(2).all(|w| {
        let w1 = w[0];
        let w2 = w[1];
        w1.0 <= w1.1 && w2.0 <= w2.1
    });
    assert!(is_valid_curve);

    let nvml = Nvml::init()?;
    let device = nvml.device_by_index(0)?;
    let num_fans = device.num_fans()?;

    loop {
        let temp = device.temperature(TemperatureSensor::Gpu)?;
        let target_speed = get_target_speed(temp, &curve);
        let current_speed = device.fan_speed(0)?;
        if target_speed == current_speed {
            std::thread::sleep(Duration::from_secs(5));
            continue;
        }
        println!("Setting fans to {}%", target_speed);
        for i in 0..num_fans {
            device.set_fan_speed(i, target_speed)?;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}

fn get_target_speed(temp: u32, curve: &[(u32, u32)]) -> u32 {
    if temp <= curve[0].0 {
        return curve[0].1;
    }
    if temp >= curve[curve.len() - 1].0 {
        return curve[curve.len() - 1].1;
    }
    for targets in curve.windows(2) {
        let (low_temp_target, _low_fan_target) = targets[0];
        let (high_temp_target, high_fan_target) = targets[1];
        if temp >= low_temp_target && temp <= high_temp_target {
            return high_fan_target;
        };
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_target_speed() {
        let curve = vec![(20, 20), (50, 50), (75, 75)];
        assert_eq!(get_target_speed(0, &curve), 20);
        assert_eq!(get_target_speed(25, &curve), 50);
        assert_eq!(get_target_speed(50, &curve), 50);
        assert_eq!(get_target_speed(60, &curve), 75);
        assert_eq!(get_target_speed(75, &curve), 75);
    }
}

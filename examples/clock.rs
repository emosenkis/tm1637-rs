extern crate chrono;
extern crate sysfs_gpio;
extern crate tm1637;

use std::env;
use std::thread::sleep;
use std::time::Duration;

use chrono::Local;
use tm1637::TM1637;
use sysfs_gpio::Pin;

fn main() {
    let mut pins = env::args()
        .skip(1)
        .take(2)
        .map(|a| Pin::new(a.parse().expect("Pins must be specified as integers")));
    let clk = pins.next().expect("Please provide two pin numbers");
    let dio = pins.next().expect("Please provide two pin numbers");
    clk.with_exported(|| {
            dio.with_exported(|| {
                let tm = try!(TM1637::new(clk.clone(), dio.clone()));
                loop {
                    let now = Local::now();
                    tm.set_string(&format!("{}", now.format("%H:%M"))).unwrap();
                    sleep(Duration::new(0,
                                        1_000_000_000 -
                                        now.timestamp_subsec_nanos() % 1_000_000_000));
                    tm.set_string(&format!("{}", now.format("%H%M"))).unwrap();
                    sleep(Duration::from_millis(500));
                }
            })
        })
        .unwrap();
}

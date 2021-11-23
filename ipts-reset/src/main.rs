use ipts_dev::{Ipts, IptsExt};

fn main() {
    Ipts::new().send_reset();
    println!("IPTS Reset");
}

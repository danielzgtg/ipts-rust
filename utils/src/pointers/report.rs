#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Report {
    None,
    Up,
    UpDown((u32, u32)),
    Down((u32, u32)),
    Move((u32, u32)),
}

#[derive(Default)]
pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn gen_id(&mut self) -> i32 {
        let count = self.count + 1;
        self.count = count;
        count
    }
}

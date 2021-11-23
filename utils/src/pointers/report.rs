#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Report {
    None,
    Up,
    UpDown((u32, u32)),
    Down((u32, u32)),
    Move((u32, u32)),
}

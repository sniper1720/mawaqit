/// Rounding rule applied to computed prayer times.
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Rounding {
    Nearest,
    Up,
    None,
}

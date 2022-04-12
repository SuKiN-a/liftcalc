#[derive(Debug, Clone, Copy)]
pub struct UnsupportedMachineError<'a>(pub &'a str);
impl std::fmt::Display for UnsupportedMachineError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.0, f)
    }
}
impl<'a> std::error::Error for UnsupportedMachineError<'a> {}

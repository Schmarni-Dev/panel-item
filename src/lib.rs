pub mod protocol;
pub mod node;
pub mod scenegraph;
pub mod panel_item;
pub mod dbus;
pub mod asteroids;

#[cfg(all(feature="acceptor", feature="provider"))]
compile_error!("the \"acceptor\" and \"provider\" features cannot be used at the same time");

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

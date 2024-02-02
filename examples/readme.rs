use map_box::Map;

pub fn main() {
    let b = Box::new(42u64);
    let _b = b.map(|v| v as i64);
}
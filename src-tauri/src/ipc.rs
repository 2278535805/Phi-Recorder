pub mod client {
    use serde::Serialize;

    pub fn send<T: Serialize>(value: T) {
        println!("{}", serde_json::to_string(&value).unwrap());
    }
}

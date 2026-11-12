pub trait Shape {
    fn area(&self) -> f64 {
        todo!()
    }

    fn name(&self) -> &str {
        unimplemented!()
    }
}

pub fn parse(input: &str) -> u32 {
    if input.is_empty() {
        todo!("empty input")
    }
    input.len() as u32
}

pub fn fail() -> ! {
    panic!("not implemented")
}

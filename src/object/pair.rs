use Object;

#[derive(Debug, PartialEq)]
pub struct Pair {
    pub car: Object,
    pub cdr: Object,
}

impl Pair {
    pub fn new(car: Object, cdr: Object) -> Self {
        Pair {
            car,
            cdr,
        }
    }
}


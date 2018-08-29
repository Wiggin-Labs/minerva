use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Pair<T: Clone>(Rc<RefCell<_Pair<T>>>);

impl<T: Clone> Pair<T> {
    pub fn new(car: T, cdr: T) -> Self {
        Pair(Rc::new(RefCell::new(_Pair::new(car, cdr))))
    }

    pub fn car(&self) -> T {
        self.0.borrow().car.clone()
    }

    pub fn cdr(&self) -> T {
        self.0.borrow().cdr.clone()
    }

    pub fn set_car(&self, car: T) {
        self.0.borrow_mut().car = car;
    }

    pub fn set_cdr(&self, cdr: T) {
        self.0.borrow_mut().cdr = cdr;
    }
}

#[derive(Debug, PartialEq)]
struct _Pair<T> {
    car: T,
    cdr: T,
}

impl<T> _Pair<T> {
    fn new(car: T, cdr: T) -> Self {
        _Pair {
            car,
            cdr,
        }
    }
}


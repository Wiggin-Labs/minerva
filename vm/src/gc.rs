use value::VType;

use std::num::NonZeroU64;
use std::lazy::SyncLazy;
use std::sync::Mutex;

pub static VMGC: SyncLazy<Mutex<Gc>> = SyncLazy::new(|| Mutex::new(Gc::new()));

pub fn get_head() -> u64 {
    VMGC.lock().unwrap().get_head()
}

pub fn set_head(p: u64, ty: VType) {
    VMGC.lock().unwrap().set_head(p, ty)
}

pub struct Gc {
    head: Option<NonZeroU64>,
}

impl Gc {
    pub fn new() -> Self {
        Gc {
            head: None
        }
    }

    pub fn get_head(&self) -> u64 {
        match self.head {
            Some(p) => p.get(),
            None => 0,
        }
    }

    pub fn set_head(&mut self, p: u64, ty: VType) {
        if p == 0 {
            self.head = None;
        } else {
            let p = (p & 0x00_FF_FF_FF_FF_FF_FF_FF) | ((ty as u64) << 56);
            self.head = Some(NonZeroU64::new(p).unwrap());
        }
    }
}

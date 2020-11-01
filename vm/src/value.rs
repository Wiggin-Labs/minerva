#![allow(non_upper_case_globals, non_snake_case)]

use string_interner::Symbol;

use std::{fmt, ops};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq)]
pub struct Value(pub u64);

// A signaling NAN constant
// The left-most bit of the significand must be 0, and at least one of the bottom 51 bits must be 1
// in order to differentiate from INF/-INF. We need the bottom 48 bits for pointers, which
// currently only use 48bits on amd64. This leaves us with 4 unused bits (48, 49, 50, and 63). Note
// that the sign bit does not matter, so we *could* use it as part of the tag.
const NAN: u64 = 0x7FF0000000000000;
const TAG_MASK: u64 = 0b111 << 48;
const IMMEDIATE_MASK: u64 = 0b1111 << 44;

// The following values need 32 bits or less, so they all share a tag of 0b000 and use some of the
// folowing bits to differentiate. This lets us pack many types under one tag.
const IMMEDIATE_TAG: u64 = 0b000 << 48;
const VOID_TAG: u64 =   0b0001 << 44;
const NIL_TAG: u64 =    0b0010 << 44;
const BOOL_TAG: u64 =   0b0011 << 44;
const INT_TAG: u64 =    0b0100 << 44;
const SYMBOL_TAG: u64 = 0b0101 << 44;
const TRUE: u64 = 1;
const FALSE: u64 = 0;

const LAMBDA_TAG: u64 = 0b001 << 48;
const PAIR_TAG: u64 =   0b010 << 48;
const VEC_TAG: u64 =    0b011 << 48;
const STRING_TAG: u64 = 0b100 << 48;


const OTHER_TAG: u64 = 0b111 << 48;

// TODO: replace middle & with && when it is allowed in const fn
macro_rules! is_imm {
    ($name:ident, $tag:ident) => {
        pub const fn $name(self) -> bool {
            ((self.0 & NAN) == NAN) & ((self.0 & TAG_MASK) == IMMEDIATE_TAG)
                & ((self.0 & IMMEDIATE_MASK) == $tag)
        }
    };
}

macro_rules! create_pointer {
    ($name:ident, $tag:ident) => {
        pub const fn $name(p: u64) -> Self {
            // We & the pointer with (2^48) - 1 because while Amd64 currently only uses the lower
            // 48bits for pointers, it requires the high 16 bits to be the same as the 48th bit.
            // For our |'s to work properly, we need the upper bits to be 0.
            Value(NAN | $tag | (p & ((1 << 48) - 1)))
        }
    };
}

// TODO: replace middle & with && when it is allowed in const fn
macro_rules! is_pointer {
    ($name:ident, $tag:ident) => {
        pub const fn $name(self) -> bool {
            ((self.0 & NAN) == NAN) & ((self.0 & TAG_MASK) == $tag)
        }
    };
}

impl Value {
    pub const Void: Self = Value(NAN | VOID_TAG);
    is_imm!(is_void, VOID_TAG);

    pub const Nil: Self = Value(NAN | NIL_TAG);
    is_imm!(is_nil, NIL_TAG);

    // TODO: make const when if is allowed
    pub fn Bool(b: bool) -> Self {
        if b { Self::True } else { Self::False }
    }
    is_imm!(is_bool, BOOL_TAG);

    pub const True: Self = Value(NAN | BOOL_TAG | TRUE);
    // TODO: make const when if is allowed
    pub fn is_true(self) -> bool {
        !self.is_false()
    }

    pub const False: Self = Value(NAN | BOOL_TAG | FALSE);
    // TODO: make const when if is allowed
    pub fn is_false(self) -> bool {
        self.is_bool() && ((self.0 & TRUE) == FALSE)
    }

    pub const fn Integer(i: i32) -> Self {
        Value(NAN | INT_TAG | (i as u32 as u64))
    }
    is_imm!(is_integer, INT_TAG);

    pub const fn to_integer(self) -> i32 {
        self.0 as u32 as i32
    }

    // TODO: make this const when const mem::transmute is stable
    pub fn Float(f: f64) -> Self {
        Value(f.to_bits())
    }

    pub const fn is_float(self) -> bool {
        (self.0 & NAN) != NAN
    }

    // TODO: make this const when const mem::transmute is stable
    pub fn to_float(self) -> f64 {
        f64::from_bits(self.0)
    }

    pub fn Symbol(s: Symbol) -> Self {
        // TODO: this should probably check that s is only 32/48 bits
        Value(NAN | SYMBOL_TAG | (*s as u64))
    }
    is_imm!(is_symbol, SYMBOL_TAG);

    pub fn to_symbol(self) -> Symbol {
        Symbol::new(self.0 as u32 as usize)
    }

    create_pointer!(Lambda, LAMBDA_TAG);
    is_pointer!(is_lambda, LAMBDA_TAG);

    create_pointer!(Pair, PAIR_TAG);
    is_pointer!(is_pair, PAIR_TAG);

    create_pointer!(Vec, VEC_TAG);
    is_pointer!(is_vec, VEC_TAG);

    create_pointer!(String, STRING_TAG);
    is_pointer!(is_string, STRING_TAG);

    // TODO: make const when if is allowed
    pub fn to_pointer(self) -> u64 {
        // Amd64 currently only uses the lower 48 bits for pointers, which is what makes NANboxing
        // possible. However, it requires that the upper 16 bits of a pointer be the same as the
        // 48th bit, so here we check whether it is 1 or 0 and set them appropriately.
        if 1 == (self.0 >> 47) & 1 {
            self.0 | (0xFFFF << 48)
        } else {
            self.0 & ((1 << 48) - 1)
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_float() {
            write!(f, "{}", self.to_float())
        } else if self.is_integer() {
            write!(f, "{}", self.to_integer())
        } else if self.is_symbol() {
            let s = self.to_symbol();
            write!(f, "{:?}", s)
        } else if *self == Value::True {
            write!(f, "#t")
        } else if *self == Value::False {
            write!(f, "#f")
        } else if self.is_void() {
            Ok(())
        } else if self.is_lambda() {
            write!(f, "#<procedure>")
        } else {
            write!(f, "debug: {:?}", self)
        }
    }
}

impl ops::Deref for Value {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value(v)
    }
}

pub mod heap_repr {
    use super::Value;
    use {Environment, Operation};

    /*
    pub enum Arity {
        Exactly(u8),
        AtLeast(u8),
    }
    */

    pub struct Lambda {
        gc: u64,
        pub env: Environment,
        arity: u8,
        pub code: ::std::vec::Vec<Operation>,
    }

    impl Lambda {
        pub fn new(root: u64, env: Environment, arity: u8, code: ::std::vec::Vec<Operation>) -> Self {
            Lambda {
                gc: root & 0xff_ffff_ffff_ffff,
                env: env,
                arity,
                code,
            }
        }
    }

    pub struct Pair {
        pub(crate) gc: u64,
        pub car: Value,
        pub cdr: Value,
    }

    impl Pair {
        pub fn new(root: u64, car: Value, cdr: Value) -> Self {
            Pair {
                // set top byte to 0 so it can be used for gc
                gc: root & 0xff_ffff_ffff_ffff,
                car,
                cdr,
            }
        }
    }

    pub struct String {
        gc: u64,
        p: ::std::string::String,
    }

    impl String {
        pub fn new(s: ::std::string::String) -> Self {
            String {
                // TODO
                gc: 0,
                p: s,
            }
        }
    }

    pub struct Vec {
        gc: u64,
        p: ::std::vec::Vec<Value>,
    }
}

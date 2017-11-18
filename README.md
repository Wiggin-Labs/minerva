# R7.rs

A scheme interpreter

### TODO
- real scheme throws error on (= 1 2 't) because of symbol. Our interpreter allows
this because it returns when it finds that 1 != 2. Is this only the = function or
are all primitives type checked?
- Support other scheme types, rat, floating point, complex, vectors
  - This mostly requires improving the parser
  - Will probably require a Number enum for generic operations
- Implement more primitives
- Support more of r7rs

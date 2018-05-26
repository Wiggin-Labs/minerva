# Akuma

A scheme interpreter

Dedicated to the Nissan Fairlady S30 240z.

### TODO
- real scheme throws error on (= 1 2 't) because of symbol. Our interpreter allows
this because it returns when it finds that 1 != 2. Is this only the = function or
are all primitives type checked?
- Support other scheme types, rat, floating point, complex, vectors
  - This mostly requires improving the parser
- Implement more primitives
- Support more of r7rs
- Add quasiquoting with unquoting
- Emulate racket when displaying quoted values
- Lazy lists
- Parallel pipelines? https://adamdrake.com/command-line-tools-can-be-235x-faster-than-your-hadoop-cluster.html
  - https://lparallel.org
- Browser interaction
  - Webdriver API
- Module system
- JIT compilation
  - Maybe look at Cretonne
  - A tiered compiler like SpiderMonkey would be interesting

#### Priority
- Macros
- Rust FFI
- REPL/environment
- Continuations
- Bytecode


#### stdlib
- HTTP lib
  - maybe look at Golang's net/http or Common Lisp's Caveman/ningle
  - Should be easy to make requests
  - Should be capable running a server, but maybe doesn't need something akin to RoR
- Database interaction
  - Should have easy drivers for common databases, Psql, SQLite, etc
  - Should have some key/value store like Golang's Boltdb
- GUI library
  - Not sure how this will come out, but the picture language in SICP has me interested
  - Look at Common Lisp's McCLIM
- Text manipulation
  - This includes things like RegEx, fuzzy matching, etc.
  - Important because plaintext is extremely common, useful as a bridge for interacting
  with UNIX tools
- De/Serialization
  - JSON is important for interacting with the web

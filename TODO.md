# TODO

- [ ] View contents of VM (registers, PC, stack)
- [ ] Step-by-step operation
- [ ] Easier switching between roms
  - CLI with first arg
  - eventually allow choosing in UI, too
- [ ] Ability to tweak "instructions per sec" so games run as expected
- [ ] Better timer solution which actually ticks 60 Hz (threads and mutexes?)
  - how to work with step-by-step operation?
- [ ] Ensure we pass the test suite
- [ ] An 8-bit sound timer which functions like the delay timer, but which also gives off a beeping sound as long as it’s not 0
- [ ] Super Chip-48 instructions http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.2
- [ ] Elegant setters/getters for registers
- [x] Include a FONT during setup
- [x] wasm build
- [x] An 8-bit delay timer which is decremented at a rate of 60 Hz (60 times per second) until it reaches 0
- [x] Make pixels fade out, giving a phosphorous CRT-style effect

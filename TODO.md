# TODO

Dev Flow:

- [ ] View contents of registers while debugging

Features:
- [ ] wasm build
- [x] An 8-bit delay timer which is decremented at a rate of 60 Hz (60 times per second) until it reaches 0
- [ ] Better timer solution which actually ticks 60 Hz (threads and mutexes?)
- [ ] An 8-bit sound timer which functions like the delay timer, but which also gives off a beeping sound as long as it’s not 0
- [x] Include a FONT during setup
- [ ] Make pixels fade out, giving a phosphorous CRT-style effect
- [ ] Super Chip-48 instructions http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.2

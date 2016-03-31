# midi_cc_convert
MIDI Control Change Event Converter.

This program convert from MIDI IN Control Change(CC) number to MIDI OUT CC number.
CC number mapping is defined commma(,) separated files.

For example:
```
10,20
74,103
```

Left value is MIDI IN CC number.
Right value is MIDI OUT CC number.

For executing this program:
```
cargo run mapping.txt
```

or

```
cargo build --release
target/release/midi_cc_convert mapping.txt
```

for more information:
```
cargo run -h
```

# midi_cc_convert
MIDI Control Change Event Converter.

This program convert from MIDI IN Control Change(CC) number to MIDI OUT CC number.
CC number mapping is defined commma(,) separated files.

For example:
```
:10,:20
16:74,1:103
```

Left value is MIDI IN CC ch:number.
Right value is MIDI OUT CC ch:number.

When MIDI IN ch is blank, it's matching all channel.
When MIDI OUT ch is blank, it's same to input channel.

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

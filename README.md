# midioscar

**Pass MIDI input out as OSC messages**

A minimal implementation of a simple application taking input from a MIDI device,
converting those to the OSC protocol and sending it out over UDP.

Idea based on [MidiOSC](https://github.com/jstutters/MidiOSC).

midioscar will convert MIDI input into an OSC message with the address: `/midi/[Device name]/[Channel]`.
The arguments will be:

* A stringified name for the action
* The first argument as an 32-bit int
* The second argument as an 32-bit int

It can take in multiple MIDI devices and push out to multiple OSC receivers.

## In production

This little tool was used in the opera play "Einstein On The Beach" by Phil Glass and Robert Wilson at the Oper Dortmund.
This work is sponsored by [rrbone](https://www.rrbone.net/) and released as open-source.

## Build

Required dependencies:

* `portmidi`
* and probably some audio lib. If you are using ALSA, you should be good to go

```
cargo build --release
```

## Usage

### List devices

```
midioscar list
```

### Serve MIDI messages as OSC

Taking input from MIDI device with ID 3.  
Pushing UDP messages to `127.0.0.1:6001`

```
midioscar serve -i 3 -h 127.0.0.1:6001
```

The options for inputs (`-i / --input`) and target hosts (`-h / --host`) can be repeated multiple times.

## License

This work is sponsored by [rrbone](https://www.rrbone.net/) and released as open-source work.

MIT License. See [LICENSE](LICENSE).

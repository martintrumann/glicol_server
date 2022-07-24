# Glicol server

This is a simple server to run local Glicol files.

Also check out the [Main Repository](https://github.com/chaosprint/glicol).

## Running Glicol
The best way to send glicol code is through the unix fifo pipe
located in `/tmp/glicol.fifo`.

single line:
```
echo o: sin 440 > /tmp/glicol.fifo
```

or entire file:
```
cat file.glicol > /tmp/glicol.fifo
```

To stop the playback use `s` `stop` or `pause`:
```
echo stop > /tmp/glicol.fifo
```

To resume use `p` or `play`:
```
echo play > /tmp/glicol.fifo
```

To change the bpm of the engine use `set_bpm` at top of the file:
```
set_bpm 66
---
o: seq 40 60 50 40 >> sawsyth 0.01 0.1
```

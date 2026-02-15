# Level

Garrett Reiter 2026

This is a Rust app for the BBC micro:bit v2, which uses the 
onboard accelerometer and LED grid to provide an indication of
its position in space relative to Earth's gravity.

## Build and Run

Instructions are provided in the embedded micro:bit Discovery Book for
setting up a build environment for the micro:bit.

from the cloned repo on the controlling computer run:
```
cargo embed --release
```
to flash and run

Another command is:
```
cargo run --release
``` 


## Observations

The modes of operation, coarse and fine, can be alternated between using button presses. 
  
Button B = Fine mode, where the level is more responsive  
Button A = Course mode, where more change is required to move the indicator  
  
Improvements can be made to this program in the future by creating MAX and MIN 
constants and using the Button input to incrementally adjust the BOUND value within 
this range.  
  
I also think that a faster cycle would be nice in fine mode; the response is currently
sluggish, and the indicator light skips around.

# Step/Dir Test Stand

## About

Test stand for Step/Dir. Currently in development.


## Hardware

The following hardware is used for the test stand:

- Adafruit full-sized breadboard
  https://www.adafruit.com/product/239
- LPC845-BRK development board
  https://www.nxp.com/products/processors-and-microcontrollers/arm-microcontrollers/general-purpose-mcus/lpc800-cortex-m0-plus-/lpc845-breakout-board-for-lpc84x-family-mcus:LPC845-BRK
- Pololu STSPIN220 stepper motor driver carrier
  https://www.pololu.com/category/260/stspin220-low-voltage-stepper-motor-driver-carriers
- Pololu NEMA 14 stepper motor
  https://www.pololu.com/product/1208
- Pololu Magnetic Encoder
  https://www.pololu.com/product/3499

The electronics go on the breadboard, the motor and encoder go on a [3D-printed structure](https://github.com/braun-embedded/step-dir/blob/master/test-stand/test-stand.scad).

### Wiring

You need to wire up the LPC845/BRK, motor driver, motor, and encoder correctly. How to do this has not been documented yet.


## Running the tests

1. Connect the LPC845-BRK to the host PC via USB.
2. From this directory, run `cargo test -p tests`.


## Known issues

The 3D-printed test stand structure has some known issues. Please be aware of those before printing it. Pull requests with improvements are very welcome!

Here's the list of known issues:

- Could use less material and time to print: I've initially designed the model to attach to the small dovetail connectors that breadboards tend to have. For this reason, it is has the length of a full-size breadboard. In the end, I ran out of time and didn't manage to get this working, meaning the model ended up bigger than it needs to be for what it does.
- Central motor mounting hole is too tight: It should be the right size in theory, but when printed without supports, the upper part of the hole droops down a bit. It shouldn't be a problem to increase the size of the whole to compensate. Printing with supports or scraping some material away with a knife should also work.
- Motor shaft attachment is too tight: This is a bit of a balancing act, as it also shouldn't sit too loose, but it came out way too tight for me. Increasing the size with a 5 mm drill bit worked great for me.
- Encoder board holder could be a tiny bit tighter. It's perfect for the board itself, but once cables are connected, those cables can easily lift the board out of the holder when the test stand is moved.

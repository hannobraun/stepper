# How to Write a Driver

## Introduction

Stepper is a software library for controlling stepper motors. To actually do that, it needs to interface with hardware that can drive the motor. To simplify things, we're going to refer to this as "driver hardware" going forward.

Stepper provides a unified interface, so the code that uses it does not need to know about the specific driver hardware being used. Under the hood, Stepper needs to know all those specifics, of course. The code that handles that for a given model of driver hardware is called a "driver".

To add support for additional driver hardware, a new driver needs to be written. The next section introduces some basics that apply to all drivers. Later sections go into the different types of driver hardware, mainly stepper drivers and motion controllers, and how to write drivers for them.


## Driver Traits

The `traits` module contains traits that can be implemented by drivers. The `Stepper` API uses these traits to abstract over drivers and provide a unified interface.

Two kinds of traits that are available:

- Traits that represent hardware capabilities. `Step` or `SetDirection` are examples of this. Your driver should implement all capability traits that can be supported by the hardware.
- Traits that allow enabling a given capability from software. For example `EnableStepControl` for `Step` or `EnableDirectionControl` for `SetDirection`. If your driver implements a capability trait, it should also implement the corresponding "enable" trait.

Drivers use a pattern called "type state" to encode, at compile-time, which capabilities are available. To provide maximum flexibility to users, Stepper makes no assumptions about which capabilities can be controlled from software in a given situation. For example, if a user uses Stepper to generate step signals, but wants to wire up a physical switch to control the direction, this should be possible.

To that end, drivers start without any capabilities. To enable a capability (e.g. setting direction) the user needs to pass the resource required to do that (e.g. the `OutputPin` implementation that controls the DIR signal) to the driver. This happens through the unified API in `Stepper`, and the various "enable" traits make this possible.

Here's an overview of all of the traits:

- `Step`/`EnableStepControl`: `Step` is a fairly low-level trait that abstracts over making single steps.
`SetDirection`/`EnableDirectionControl`: `SetDirection` controls the direction of steps made with `Step`. Implementing it only makes sense, if the driver implements `Step`.
- `SetStepMode`/`EnableStepModeControl`: Microstepping is a technique for more fine-grained control of stepper motors. Most driver hardware seems to support it these days, but some might not. Some driver hardware has physical switches to control microstepping configuration, meaning that it can't be changed from software.
- `MotionControl`/`EnableMotionControl`: `MotionControl` abstracts over high-level motion control capability, for example moving a specific number of steps while smoothly accelerating/deceleration to/from the maximum velocity.

Your driver should implement all traits whose capabilities the hardware can support. The following sections have some more notes on what that might look like for different kinds of driver hardware.


## Types of Driver Hardware

### Stepper Drivers

Lots of driver hardware implements a standard interface, consisting of digital STEP and DIR signals. Those allow for making steps and controlling direction. This type of driver hardware is often called "stepper driver".

Stepper drivers tend to be fairly low-level, so their drivers typically only need to implement the lower-level traits:

- `Step`/`EnableStepControl` and `SetDirection`/`EnableDirectionControl`: This is an ubiquitous capability with stepper drivers. If driver hardware has STEP and DIR signals, it should implement these traits.
- `SetStepMode`/`EnableStepModeControl`: If the hardware supports microstepping, and that configuration can be controlled from software, the driver should implement these traits.
- `MotionControl`/`EnableMotionControl`: Typical stepper drivers don't have motion control capability and can't support these traits natively. A software-based fallback implementation based on `Step` and `SetDirection` is available, but as a driver author, you don't have to worry about that.

Please note that some driver hardware is a hybrid between a typical stepper driver (i.e. it provides STEP and DIR signals) and a higher-level motion controller. If you're faced with hardware like this, you can implement support for its low-level features, as laid out in this section. The next section goes into how to support motion control capability.

### Motion Controllers

The defining feature of motion controllers is that they provide a high-level motion control interface that allows for moving a specific number of steps, or at a specific speed. They also provide smooth acceleration between different speeds. This is functionality that would otherwise have to be implemented in software, if using just a low-level stepper driver.

As of this writing, Stepper does not support any motion controllers. However, the `MotionControl` trait was written with them in mind. Hopefully, it should be possible to implement `MotionControl` for any motion controllers, with little to no changes to the trait required.

Since motion controllers should be able to make a single step, it should be possible to implement `Step`/`SetDirection` for them too. Whether that is desirable is a subject for future exploration. `SetStepMode`/`EnableStepModeControl` should be implemented, if the hardware can support them.

### Other Hardware

There is more driver hardware that doesn't fall neatly into one of the two groups (stepper drivers and motion controllers) outlined above. At this point, Stepper doesn't support any of them. Whether they can be made to fit the existing traits, or wether modifications or entirely new traits are required, are open questions.


## Writing a Driver

### Driver Module

When starting to write a new driver, start with copying an existing one. All existing drivers are located in `src/drivers/`, with each having a separate file. Choose one that you think is most similar to what you're going to need (if in doubt, pick `drv8825.rs`).

As an aside, it would be nice if there was less duplicated code between the drivers. A lot has already been done to reduce this, by simplifying the driver traits and moving non-trivial code out of the drivers, but a lot of boilerplate code for managing driver state remains. It should be possible to reduce this to a bare minimum using procedural macros, but that remains an area for future exploration.

Go through the driver you copied, and change it as required. Most changes should fall into one of the following categories:

- Update names that refer to hardware (the stepper driver itself and its pins).
- Make sure the timing constants match the required timing, as per the documentation of the stepper driver.
- Make sure all traits that the stepper driver can support are implemented.

The last item is the least trivial, as different driver hardware has different capabilities, and thus allow the driver to implement a different set of traits. Please use the information on the traits from the previous section as a guideline.

### Driver Crate

In addition to the modules in Stepper itself, drivers can be used via external crates that re-export the relevant parts of Stepper. This has been done to provide more convenience to users that only need support for a specific driver, and to make it easier to discover Stepper through the driver hardware it supports.

To create an external driver crate, add an entry to `drivers.toml`, then run the `generate-drivers` task to generate the code for it:

``` bash
cargo install cargo-task # only required once
cargo task generate-drivers
```

This should generate a new driver crate in `drivers/` from the template in `templates/driver/`. Please note that the template is fairly specific to the currently existing drivers, and it might be necessary to adapt it for new ones.

If in doubt, feel free to skip creating the driver crate. The important part is having the driver in Stepper. An external crate can always be created later.

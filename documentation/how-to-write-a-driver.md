# How to Write a Driver

## Introduction

Stepper is a software library for controlling stepper motors. To actually do that, it needs to interface with hardware that can drive the motor. To simplify things, we're going to refer to this as "driver hardware" going forward.

Stepper provides a unified interface, so the code that uses it does not need to know about the specific driver hardware being used. But under the hood, Stepper needs to know all those specifics, of course. Code that does that for a given model of driver hardware is called a "driver".

To add support for new driver hardware, a new driver needs to be written. The rest of this guide explains how to do that.


## Stepper Drivers

Lots of driver hardware implements a standard interface, consisting of digital STEP and DIR signals. Those allow for making steps and control direction. This type of driver hardware is often called "stepper driver".

Adding support for more stepper drivers should be relatively straight-forward, as their code ends up being very similar. Existing drivers are located in `src/drivers/`, with each driver having a separate file. Choose one of them that you think is going to be similar to the driver you are going to write (if in doubt, choose `drv8825.rs`), and make a copy of it.

As an aside, it would be nice if there was less duplicated code between the drivers. A lot has already been done to reduce this, by simplifying the driver traits and moving non-trivial code out of the drivers, but a lot of boilerplate code for managing driver state remains. It should be possible to reduce this to a bare minimum using procedural macros, but that remains an area for future exploration.

Go through the driver you copied, and change it as required. Most changes should fall into one of the following categories:

- Update names that refer to hardware (the stepper driver itself and its pins).
- Make sure the timing constants match the required timing, as per the documentation of the stepper driver.
- Make sure all traits that the stepper driver can support are implemented.

The last item is the least trivial, as different stepper drivers have different capabilities, and thus allow the driver to implement a different set of traits. Before we go into that, let's take a look at the two kinds of traits that are available:

- Traits that represent hardware capabilities. `Step` or `SetDirection` are examples of this. Your driver should implement all capability traits that can be supported by the hardware.
- Traits that allow enabling a given capability from software. For example `EnableStepControl` for `Step` or `EnableDirectionControl` for `SetDirection`. If your driver implements a capability trait, it should also implement the corresponding "enable" trait.

Drivers use a pattern called "type state" to encode, at compile-time, which capabilities are available. To provide maximum flexibility to users, Stepper makes no assumptions about which capabilities can be controlled from software in a given situation. For example, if a user uses Stepper to generate step signals, but wants to wire in a physical switch to control the direction, this should be possible.

To that end, drivers start without any capabilities. To enable a capability (e.g. setting direction) the user needs to pass the resource required to do that (e.g. the `OutputPin` implementation that controls the DIR signal) to the driver. This happens through the unified API in `Stepper`, and the various "enable" traits make this possible.

Now let's take a look at which traits exist, and under which circumstances they should be implemented:

- `Step`/`EnableStepControl` and `SetDirection`/`EnableDirectionControl`: `Step` and `SetDirection` control making steps and setting the direction signal. Since these are the most basic tasks of stepper drivers, it should be possible to implement them in virtually any driver.
- `SetStepMode`/`EnableStepModeControl`: Microstepping is a technique for more fine-grained control of stepper motors. Most driver hardware seems to support it these days, but some might not. Also, with some stepper drivers, the microstepping mode is configured using physical switches, meaning that configuration can't be changed from software. A driver should implement those traits, if the driver hardware supports microstepping and allows for configuring it from software.
- `MotionControl`/`EnableMotionControl`: Lastly, there is high-level motion control capability. Typical stepper drivers don't support this natively, and for those there is a software implementation of `MotionControl` based on `Step` and `SetDirection`. As a driver author, you don't have to worry about that.

Please note that some driver hardware is a hybrid between a typical stepper driver (i.e. it provides STEP and DIR signals) and a higher-level motion controller. If you're faced with hardware like this, you can implement support for its low-level features, as laid out in this section. The next section goes into how to write drivers for high-level motion controllers.


## Motion Controllers

The previous section went into the basics of drivers in Stepper, and explained how to write drivers for stepper drivers specifically. Please make sure to read it, as the basic information provided is relevant when writing drivers for motion controllers too.

The defining feature of motion controllers is that they provide a high-level motion control interface that allows for moving a specific number of steps, or at a specific speed. They also provide smooth acceleration between different speeds. This is functionality that would otherwise have to be implemented in software, if using just a low-level stepper driver.

As of this writing, Stepper does not support any motion controllers. However, the `MotionControl` trait was written with them in mind. Hopefully, it should be possible to implement `MotionControl` for any motion controllers, with little to no changes to the trait required.

Since motion controllers should be able to make a single step, it should be possible to implement `Step`/`SetDirection` for them too. Whether that is desirable is a subject for future exploration.


## Other Hardware

There is more driver hardware that doesn't fall neatly into one of the two groups (stepper drivers and motion controllers) described above. At this point, Stepper doesn't support any of them. Whether they can be made to fit the existing traits, or wether modifications or entirely new traits are required, are open questions.

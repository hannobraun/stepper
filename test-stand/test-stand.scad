// Please refer to README.md for links to the hardware that is referenced here.

// Dimensions of the stepper motor
motor_width  = 35.2;
motor_length = 28.0;
motor_shaft  = 21.0;

// Motor holder dimensions
motor_holder_clearance    = 2.5;
motor_holder_walls        = 2.0;
motor_holder_width_inner  = motor_width + 2 * motor_holder_clearance;
motor_holder_width_outer  = motor_holder_width_inner + 2 * motor_holder_walls;
motor_holder_height_outer = motor_holder_width_outer;
motor_holder_length_inner = motor_length + motor_holder_clearance;
motor_holder_length_outer = motor_holder_length_inner + motor_holder_walls;
motor_holder_shaft_offset = motor_width / 2;
motor_holder_holes_offset = motor_holder_walls + motor_holder_clearance;
motor_holder_shaft_pos    = motor_holder_holes_offset+motor_holder_shaft_offset;

// Dimensions of the encoder board
encoder_width           = 20.0;
encoder_pcb_height      =  1.02;
encoder_bottom_to_shaft =  5.7;
encoder_total_height    = encoder_pcb_height + 1.2;
encoder_magnet_height   = 2.0;

// Dimensions of the encoder holder
encoder_holder_walls  = motor_holder_walls;
encoder_holder_width  = encoder_width + 2 * encoder_holder_walls + 0.6;
encoder_holder_length = encoder_pcb_height + 2 * encoder_holder_walls;
encoder_holder_base   = motor_holder_shaft_pos - encoder_bottom_to_shaft;

// Dimensions of the shaft that connects motor and magnet
shaft_hole_length   = 15.0;
shaft_body_length   =  5.0;
shaft_magnet_length =  encoder_magnet_height * 2;

// Distance between the motor and encoder holders. Doesn't use the full depth of
// the hole in the shaft to provide some wiggle room.
holders_distance = motor_shaft - motor_holder_walls
    + shaft_hole_length * 0.5 + shaft_body_length + encoder_magnet_height
    - encoder_total_height
    + encoder_holder_length - encoder_holder_walls;

// Base dimensions that match the relevant dimensions of Adafruit's full sized
// breadboard.
base_length = 165.1;
base_height =   9.5;

// Base dimensions that are independent of breadboard dimensions.
base_side_dist = 2.5;
base_width     = motor_holder_width_outer + 2 * base_side_dist;


test_stand();

// Put the shaft off the side, so it's printed separately.
translate([-10.0, 0.0, 0.0])
shaft();


module test_stand() {
    base_end_dist_total = base_length
        - motor_holder_length_outer
        - encoder_holder_length
        - holders_distance;
    base_end_dist = base_end_dist_total / 2;

    union() {
        base();

        translate([
            (base_width - encoder_holder_width) / 2,
            base_end_dist,
            base_height
        ])
        encoder_holder();

        translate([
            base_side_dist,
            base_length - motor_holder_length_outer - base_end_dist,
            base_height
        ])
        motor_holder();
    }
}


// Base plate
//
// Can be attached to a full size breadboard.
module base() {
    y_offsets = [14.9, 82.4, 149.9];

    difference() {
        union () {
            cube([base_width, base_length, base_height]);

            for (y = y_offsets) {
                translate([0.0, y, 0.0])
                dovetail(0.05);
            }
        }

        for (y = y_offsets) {
            translate([base_width, y, 0.0])
            dovetail(-0.05);
        }
    }

    // The tiny dovetail connectors that breadboards often have.
    module dovetail(clearance) {
        outer  = 4.2 - clearance;
        inner  = 4.0 - clearance;
        depth  = 1.3 - clearance;
        height = 5.2 - clearance;

        rotate([0, 0, 90])
        linear_extrude(height)
        polygon([
            [-inner / 2, 0.0],
            [ inner / 2, 0.0],
            [ outer / 2, depth],
            [-outer / 2, depth]
        ]);
    }
}

// Holder for the NEMA 14 stepper motor
module motor_holder() {
    union() {
        front();

        translate([0.0, motor_holder_walls, 0.0])
        side();

        translate([
            motor_holder_width_outer - motor_holder_walls,
            motor_holder_walls,
            0.0
        ])
        side();
    }

    module front() {
        difference() {
            cube([
                motor_holder_width_outer,
                motor_holder_walls,
                motor_holder_height_outer
            ]);

            translate([0.0, motor_holder_walls, 0.0])
            rotate([90.0, 0.0, 0.0])
            linear_extrude(motor_holder_walls)
            translate([
                motor_holder_holes_offset,
                motor_holder_holes_offset,
                0.0
            ])
            holes();
        }
    }

    module holes() {
        screw_hole_distance = 26.0;
        screw_hole_offset_n = (motor_width - screw_hole_distance) / 2;
        screw_hole_offset_f = screw_hole_offset_n + screw_hole_distance;

        screw_hole_positions = [
            [screw_hole_offset_n, screw_hole_offset_n],
            [screw_hole_offset_n, screw_hole_offset_f],
            [screw_hole_offset_f, screw_hole_offset_n],
            [screw_hole_offset_f, screw_hole_offset_f]
        ];
        for (pos = screw_hole_positions) {
            translate([pos[0], pos[1], 0.0])
            circle2(d = 4.5);
        }

        translate([motor_holder_shaft_offset, motor_holder_shaft_offset, 0.0])
        circle2(d = 22);
    }

    module side() {
        rotate([90.0, 0.0, 90.0])
        linear_extrude(motor_holder_walls)
        polygon([
            [0.0, 0.0],
            [motor_holder_length_inner, 0.0],
            [0.0, motor_holder_height_outer]
        ]);
    }
}

// Holder for the encoder PCB
module encoder_holder() {
    cube([encoder_holder_width, encoder_holder_length, encoder_holder_base]);

    translate([0.0, 0.0, encoder_holder_base])
    union () {
        holder();

        translate([encoder_holder_width, 0.0, 0.0])
        mirror([1, 0, 0])
        holder();
    }

    module holder() {
        height = 11.4;
        depth  =  1.8;

        cube([encoder_holder_walls, encoder_holder_length, height]);

        translate([encoder_holder_walls, 0.0, 0.0])
        union() {
            front_back();

            translate([0.0, encoder_holder_length - encoder_holder_walls, 0.0])
            front_back();
        }

        module front_back() {
            cube([depth, encoder_holder_walls, height]);
        }
    }
}

// The shaft that connects the motor shaft to the encoder magnet
//
// This is not actually a fixed part of this, but it's in this model for ease of
// printing.
module shaft() {
    motor_shaft_d = 5.0;
    outer_d       = motor_shaft_d + 2 * 1.5;

    // Fits on motor shaft
    difference() {
        cylinder2(d = outer_d,       h = shaft_hole_length);
        cylinder2(d = motor_shaft_d, h = shaft_hole_length);
    }

    // Main body
    translate([0.0, 0.0, shaft_hole_length])
    cylinder2(d = outer_d, h = shaft_body_length);

    // Fits the magnet
    translate([0.0, 0.0, shaft_hole_length + shaft_body_length])
    cylinder2(d = 2.0, h = 2.0);
}


// Change this to a low value during development. Set it to 360 before printing.
fn = 360;

// A circle that is actually round
module circle2(d) {
    circle(d = d, $fn = fn);
}

// A cylinder that is actually round
module cylinder2(d, h) {
    cylinder(d = d, h = h, $fn = fn);
}

This implementation uses a simple text command protocol over serial:

    Command Format: M<motor_id>,<steps>,<direction>\n

    Example: 1,800,1\n moves Motor 1 for 800 steps clockwise. (direction: 1 = Clockwise, 0 = Counter-clockwise).

    Response: Returns DONE: <steps_moved> or STOPPED: <steps_moved> if the hard stop trips.

# test method from bash
echo "1,1000,1" > /dev/ttyACM0


# see if device shows up
ls -l /dev/ttyACM*

# build setup
cargo install elf2uf2-rs



# minimal
cargo build --release --target thumbv6m-none-eabi
elf2uf2-rs ../target/thumbv6m-none-eabi/release/pico-minimal ../target/thumbv6m-none-eabi/release/pico-minimal.uf2
cp ../target/thumbv6m-none-eabi/release/pico-minimal.uf2 /media/$USER/RPI-RP2/


# full version
cargo build --release --target thumbv6m-none-eabi
elf2uf2-rs ../target/thumbv6m-none-eabi/release/pico-stepper-controller ../target/thumbv6m-none-eabi/release/pico-stepper-controller.uf2
cp ../target/thumbv6m-none-eabi/release/pico-stepper-controller.uf2 /media/$USER/RPI-RP2/




rustup target add thumbv6m-none-eabi
# this should push to pico
cargo build --release --target thumbv6m-none-eabi



# see if it flashed
cat /media/$USER/RPI-RP2/INFO_UF2.TXT




Step 3: Put the Pico in Bootloader Mode and Run

    Unplug your Pico's USB cable.

    Press and hold down the white physical BOOTSEL button on the Pico.

    Plug the USB cable back into your computer while holding the button, then release it. (The Pico will mount itself as a virtual USB flash drive named RPI-RP2).

    In your project terminal, simply run:
    Bash

    cargo run --release

elf2uf2-rs will catch the connected Pico, upload the compiled firmware, and the Pico will instantly reboot out of bootloader mode and start running your stepper controller code! From then on, whenever you change your code, just drop the Pico into BOOTSEL mode and type cargo run --release again.
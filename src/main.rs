//! # Seeeduino XIAO RP2040 Blinky Example
//!
//! Blinks the LED on a Seeeduino XIAO RP2040 16MB board.
//!
//! This will blink an LED attached to GPIO25, which is the pin the XIAO RP2040
//! uses for the on-board LED.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

use gc9a01::display::DisplayResolution240x240;
use gc9a01::mode::BufferedGraphics;
use gc9a01::rotation::DisplayRotation;
use gc9a01::Gc9a01;
use gc9a01::SPIDisplayInterface;
// The macro for our start-up function
use seeeduino_xiao_rp2040::entry;

// GPIO traits
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_probe as _;

// Pull in any important traits
use seeeduino_xiao_rp2040::hal::prelude::*;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use seeeduino_xiao_rp2040::hal::pac;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use seeeduino_xiao_rp2040::hal;

use hal::gpio::PinState;

// DEFMT for debug logging over RTT
use defmt::*;
use defmt_rtt as _;

// The minimum PWM value (i.e. LED brightness) we want
const LOW: u16 = 0;

// The maximum PWM value (i.e. LED brightness) we want
const HIGH: u16 = 60000;

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then blinks the LED in an
/// infinite loop.
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let interface = SPIDisplayInterface::new(spi, dc, cs);
    let mut display_driver = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    );
}

// End of file

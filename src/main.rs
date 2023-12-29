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

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size},
    primitives::{Circle, Primitive, PrimitiveStyleBuilder, Rectangle},
    Drawable,
};

use gc9a01::{
    display::DisplayResolution240x240, mode::DisplayConfiguration, rotation::DisplayRotation,
    Gc9a01, SPIDisplayInterface,
};

use seeeduino_xiao_rp2040::hal;
use seeeduino_xiao_rp2040::Pins;
// The macro for our start-up function
use seeeduino_xiao_rp2040::entry;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_probe as _;

use hal::gpio::FunctionSioOutput;
use hal::spi::Spi;
use hal::Sio;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use fugit::RateExtU32;
use hal::pac;
// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use hal::clocks::Clock;

// DEFMT for debug logging over RTT
use defmt::*;
use defmt_rtt as _;

/// Test Function : will be removed later
/*
fn draw<I: WriteOnlyDataCommand, D: DisplayDefinition>(
    display: &mut Gc9a01<I, D, BufferedGraphics<D>>,
    tick: u32,
) {
    let (w, h) = display.dimensions();
    let w = w as u32;
    let h = h as u32;
    let x = tick % w;
    let y = tick % h;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(4)
        .stroke_color(Rgb565::new(tick as u8, x as u8, y as u8))
        .fill_color(Rgb565::RED)
        .build();

    let cdiameter = 20;

    // circle
    Circle::new(
        Point::new(119 - cdiameter / 2 + 40, 119 - cdiameter / 2 + 40),
        cdiameter as u32,
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // circle
    Circle::new(
        Point::new(119 - cdiameter / 2 - 40, 119 - cdiameter / 2 + 40),
        cdiameter as u32,
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // rectangle
    let rw = 80;
    let rh = 20;
    Rectangle::new(
        Point::new(119 - rw / 2, 119 - rh / 2 - 40),
        Size::new(rw as u32, rh as u32),
    )
    .into_styled(style)
    .draw(display)
    .unwrap();
}
*/
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
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    info!("Got watchdog");

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        seeeduino_xiao_rp2040::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    info!("Got clocks");
    let mut timer = hal::timer::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let spi_mosi = pins.mosi.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.miso.into_function::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.sck.into_function::<hal::gpio::FunctionSpi>();

    let spi: Spi<hal::spi::Disabled, pac::SPI0, (_, _, _), 8u8> =
        Spi::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));
    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let dc = pins.a3.into_function::<FunctionSioOutput>();
    let cs = pins.a1.into_function::<FunctionSioOutput>();
    let interface = SPIDisplayInterface::new(spi, dc, cs);
    /*
    let data = [0u8, 1u8, 0x0fu8, 0xaa, 0x55u8, 0x80u8, 0xffu8];

    loop {
        let xx: DataFormat = DataFormat::U8(&data);
        let err = interface.send_commands(xx);
        match err {
            Ok(_) => debug!("OK"),
            Err(_) => debug!("Error"),
        }
    }
    */
    let mut display_driver = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics();

    // let mut reset = pins.sda.into_function::<FunctionSioOutput>();
    //    display_driver.reset(&mut reset, &mut timer).ok();
    display_driver.init(&mut timer).ok();
    defmt::debug!("Driver configured!");

    let mut tick: u32 = 0;
    loop {
        // debug!("Tick {}", tick);
        display_driver.clear();
        let (w, h) = display_driver.dimensions();
        let w = w as u32;
        let h = h as u32;
        let x = tick % w;
        let y = tick % h;

        let style = PrimitiveStyleBuilder::new()
            .stroke_width(4)
            .stroke_color(Rgb565::new(tick as u8, x as u8, y as u8))
            .fill_color(Rgb565::RED)
            .build();

        let cdiameter = 20;

        // circle
        Circle::new(
            Point::new(119 - cdiameter / 2 + 40, 119 - cdiameter / 2 + 40),
            cdiameter as u32,
        )
        .into_styled(style)
        .draw(&mut display_driver)
        .unwrap();

        // circle
        Circle::new(
            Point::new(119 - cdiameter / 2 - 40, 119 - cdiameter / 2 + 40),
            cdiameter as u32,
        )
        .into_styled(style)
        .draw(&mut display_driver)
        .unwrap();

        // rectangle
        let rw = 80;
        let rh = 20;
        Rectangle::new(
            Point::new(119 - rw / 2, 119 - rh / 2 - 40),
            Size::new(rw as u32, rh as u32),
        )
        .into_styled(style)
        .draw(&mut display_driver)
        .unwrap();
        display_driver.flush().unwrap();
        tick += 1;
    }
}

// End of file

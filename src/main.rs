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

use embedded_graphics::{image::Image, prelude::Point, Drawable};

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

use embedded_sdmmc::{Mode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use tinyqoi::Qoi;

mod qoiwrapper;

fn panic_loop(err: &str) -> ! {
    error!("PANIC --- {}", err);
    loop {}
}

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

    // Set up the SPI bus
    let spi_mosi = pins.mosi.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.miso.into_function::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.sck.into_function::<hal::gpio::FunctionSpi>();
    let spi: Spi<hal::spi::Disabled, pac::SPI0, (_, _, _), 8u8> =
        Spi::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16.MHz(),
        embedded_hal::spi::MODE_0,
    );

    // Bus manager so we can share the bus across devices
    let bus = shared_bus::BusManagerSimple::new(spi);

    let dc = pins.a3.into_function::<FunctionSioOutput>();
    let cs = pins.a1.into_function::<FunctionSioOutput>();
    let interface = SPIDisplayInterface::new(bus.acquire_spi(), dc, cs);
    let mut display_driver = Gc9a01::new(
        interface,
        DisplayResolution240x240,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics();
    display_driver.init(&mut timer).ok();
    defmt::debug!("Driver configured!");

    defmt::debug!("Connecting to SdCard");
    let sd_cs = pins.a2.into_function::<FunctionSioOutput>();
    let card = SdCard::new(bus.acquire_spi(), sd_cs, timer);
    /// A dummy timesource, which is mostly important for creating files.
    #[derive(Default)]
    pub struct DummyTimesource();

    impl TimeSource for DummyTimesource {
        // In theory you could use the RTC of the rp2040 here, if you had
        // any external time synchronizing device.
        fn get_timestamp(&self) -> Timestamp {
            Timestamp {
                year_since_1970: 0,
                zero_indexed_month: 0,
                zero_indexed_day: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }
        }
    }
    let mut volume_mgr = VolumeManager::new(card, DummyTimesource::default());
    defmt::debug!("Volume managers created");

    let volume = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            error!("Error opening root volume: {}", defmt::Debug2Format(&e));
            panic_loop("Open volume");
        }
    };

    let dir = match volume_mgr.open_root_dir(volume) {
        Ok(dir) => dir,
        Err(e) => {
            error!("Error opening root dir: {}", defmt::Debug2Format(&e));
            panic_loop("Open root dir");
        }
    };

    debug!("Root directory opened");
    let file = match volume_mgr.open_file_in_dir(dir, "eye.qoi", Mode::ReadOnly) {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening image file: {}", defmt::Debug2Format(&e));
            panic_loop("Open image file");
        }
    };
    let mut image_buf = [0u8; 128 * 1024];
    let read_count = volume_mgr.read(file, &mut image_buf).unwrap();
    volume_mgr.close_file(file).unwrap();
    debug!("Read {} bytes from eye.qoi", read_count);

    debug!("Image data length {}", image_buf.len());

    let qoi = match Qoi::new(&image_buf[..read_count]) {
        Ok(qoi) => qoi,
        Err(e) => {
            error!("Error processing qoi file: {}", defmt::Debug2Format(&e));
            panic_loop("Qoi format");
        }
    };
    let wrapper = qoiwrapper::Wrapper { image: &qoi };
    display_driver.clear();
    Image::new(&wrapper, Point::zero())
        .draw(&mut display_driver)
        .unwrap();
    display_driver.flush().unwrap();
    debug!("Drawn image");

    loop {}
}

// End of file

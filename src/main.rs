#![no_main]
#![no_std]

mod controls; 

//pub static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));

//pub use init::init_buttons;
use controls::init_buttons;
use cortex_m_rt::{
    entry,
    //interrupt::free as interrupt_free,
};
use core::cell::RefCell;
use panic_rtt_target as _;
use critical_section::Mutex;
use critical_section_lock_mut::LockMut;
use rtt_target::{rtt_init_print, rprintln};

use microbit::{
    hal::{
        twim, 
        Timer,
        gpiote::Gpiote,
        pac::{self,interrupt},
    },
    display::blocking::Display,
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{AccelMode, AccelOutputDataRate, Lsm303agr};

//static GPIOTE_PERIPHERAL: LockMut<Gpiote> = LockMut::new();
static GPIO: Mutex<RefCell<Option<Gpiote>>> = Mutex::new(RefCell::new(None));
static BOUND: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(400));

const FRAME: u32 = 200;
//static BOUND: i32 = 400;
const EMPTY: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

#[interrupt]
fn GPIOTE() {
    //let mut bound = critical_section::with(|cs| {
    //   *BOUND.borrow(cs).borrow()
    //});

    critical_section::with(|cs| {
        if let Some(gpiote) = GPIO.borrow(cs).borrow().as_ref() {
            let a_pressed = gpiote.channel0().is_event_triggered();
            let b_pressed = gpiote.channel1().is_event_triggered();
            
            //match (a_pressed, b_pressed) {
            //    (true, false) => { 
            //        *BOUND.borrow(cs).borrow_mut() = 1000;
            //        rprintln!("Set BOUND to 1000");
            //    }
            //    (false, true) => { 
            //        *BOUND.borrow(cs).borrow_mut() = 500;
            //        rprintln!("Set BOUND to 500");
            //    }
            //    _ => {},
            let mut bound = BOUND.borrow(cs).borrow_mut();
            let _dir = match (a_pressed, b_pressed) {
                (true, false) => { *bound = 1200; }
                (false, true) => { *bound = 400; }
                _ => {},
            };
        gpiote.channel0().reset_events();
        gpiote.channel1().reset_events();

        }
            else {
                rprintln!("GPIOTE interrupt but GPIO not initialized!");
            }
    });
}

fn get_value(x: &i32) -> isize {
    let mut coord: isize;
    let bound = critical_section::with(|cs| {
       *BOUND.borrow(cs).borrow()
    });
    let interval = bound / 5;
    if *x > -interval && *x < interval {
        coord = 0;
    }
    else if *x > (-3 * interval) && *x < (3 * interval) {
        coord = 1;
    }
    else { coord = 2 }

    if *x > 0 { coord = -coord }
    coord
}

fn move_bubble(bubble: &(isize, isize), leds: &mut [[u8; 5]; 5]) {
    let x = usize::try_from(bubble.0 + 2).unwrap();
    let y = usize::try_from(bubble.1 + 2).unwrap();

    *leds = EMPTY;
    leds[y][x] = 1;
}
    

#[entry]
fn init() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    //let button_a = board.buttons.button_a.into_floating_input(); 
    //let gpiote = gpiote::Gpiote::new(board.GPIOTE);
    //let channel = gpiote.channel0();
    //channel
        //.input_pin(&button_a.degrade())
        //.hi_to_lo()
        //.enable_interrupt();
    //channel.reset_events();
    //GPIOTE_PERIPHERAL.init(gpiote);
    init_buttons(board.GPIOTE, board.buttons);

    let mut leds = [
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 1, 0],
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0],
    ]; 

    let mut bubble: (isize, isize) = ( 0, 0 );


    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut timer,
            AccelMode::HighResolution,
            AccelOutputDataRate::Hz50,
        )
        .unwrap();

    unsafe { pac::NVIC::unmask(pac::Interrupt::GPIOTE) }; 
    pac::NVIC::unpend(pac::Interrupt::GPIOTE);

    loop {
        if sensor.accel_status().unwrap().xyz_new_data() {
            let (x, y, z) = sensor.acceleration().unwrap().xyz_mg();
            // RTT instead of normal print
            rprintln!("Acceleration: x {} y {} z {}", x, y, z);
            bubble.0 = get_value(&x);
            bubble.1 = -get_value(&y);
            move_bubble(& bubble, &mut leds);

            if z < 0 {
                display.show(&mut timer, leds, FRAME);
            }
            else { display.clear(); }
        }
    }
}

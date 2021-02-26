// cargo run --example rtt_rtic_usb_mouse --release
//
// Notice, release build required

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m::{asm::delay, peripheral::DWT};
use embedded_hal::digital::v2::OutputPin;
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32f4xx_hal::{
    gpio,
    otg_fs::{UsbBus, UsbBusType, USB},
    prelude::*,
};
use usb_device::bus;
use usb_device::prelude::*;

#[allow(unused)]
pub mod hid {
    use usb_device::class_prelude::*;
    use usb_device::Result;

    pub const USB_CLASS_HID: u8 = 0x03;

    const USB_SUBCLASS_NONE: u8 = 0x00;
    const USB_SUBCLASS_BOOT: u8 = 0x01;

    const USB_INTERFACE_NONE: u8 = 0x00;
    const USB_INTERFACE_KEYBOARD: u8 = 0x01;
    const USB_INTERFACE_MOUSE: u8 = 0x02;

    const REQ_GET_REPORT: u8 = 0x01;
    const REQ_GET_IDLE: u8 = 0x02;
    const REQ_GET_PROTOCOL: u8 = 0x03;
    const REQ_SET_REPORT: u8 = 0x09;
    const REQ_SET_IDLE: u8 = 0x0a;
    const REQ_SET_PROTOCOL: u8 = 0x0b;

    // https://docs.microsoft.com/en-us/windows-hardware/design/component-guidelines/mouse-collection-report-descriptor
    const REPORT_DESCR: &[u8] = &[
        0x05, 0x01, // USAGE_PAGE (Generic Desktop)
        0x09, 0x02, // USAGE (Mouse)
        0xa1, 0x01, // COLLECTION (Application)
        0x09, 0x01, //   USAGE (Pointer)
        0xa1, 0x00, //   COLLECTION (Physical)
        0x05, 0x09, //     USAGE_PAGE (Button)
        0x19, 0x01, //     USAGE_MINIMUM (Button 1)
        0x29, 0x03, //     USAGE_MAXIMUM (Button 3)
        0x15, 0x00, //     LOGICAL_MINIMUM (0)
        0x25, 0x01, //     LOGICAL_MAXIMUM (1)
        0x95, 0x03, //     REPORT_COUNT (3)
        0x75, 0x01, //     REPORT_SIZE (1)
        0x81, 0x02, //     INPUT (Data,Var,Abs)
        0x95, 0x01, //     REPORT_COUNT (1)
        0x75, 0x05, //     REPORT_SIZE (5)
        0x81, 0x03, //     INPUT (Cnst,Var,Abs)
        0x05, 0x01, //     USAGE_PAGE (Generic Desktop)
        0x09, 0x30, //     USAGE (X)
        0x09, 0x31, //     USAGE (Y)
        0x15, 0x81, //     LOGICAL_MINIMUM (-127)
        0x25, 0x7f, //     LOGICAL_MAXIMUM (127)
        0x75, 0x08, //     REPORT_SIZE (8)
        0x95, 0x02, //     REPORT_COUNT (2)
        0x81, 0x06, //     INPUT (Data,Var,Rel)
        0xc0, //   END_COLLECTION
        0xc0, // END_COLLECTION
    ];

    pub fn report(x: i8, y: i8) -> [u8; 3] {
        [
            0x00,    // button: none
            x as u8, // x-axis
            y as u8, // y-axis
        ]
    }

    pub struct HIDClass<'a, B: UsbBus> {
        report_if: InterfaceNumber,
        report_ep: EndpointIn<'a, B>,
    }

    impl<B: UsbBus> HIDClass<'_, B> {
        /// Creates a new HIDClass with the provided UsbBus and max_packet_size in bytes. For
        /// full-speed devices, max_packet_size has to be one of 8, 16, 32 or 64.
        pub fn new(alloc: &UsbBusAllocator<B>) -> HIDClass<'_, B> {
            HIDClass {
                report_if: alloc.interface(),
                report_ep: alloc.interrupt(8, 10),
            }
        }

        pub fn write(&mut self, data: &[u8]) {
            self.report_ep.write(data).ok();
        }
    }

    impl<B: UsbBus> UsbClass<B> for HIDClass<'_, B> {
        fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
            writer.interface(
                self.report_if,
                USB_CLASS_HID,
                USB_SUBCLASS_NONE,
                USB_INTERFACE_MOUSE,
            )?;

            let descr_len: u16 = REPORT_DESCR.len() as u16;
            writer.write(
                0x21,
                &[
                    0x01,                   // bcdHID
                    0x01,                   // bcdHID
                    0x00,                   // bCountryCode
                    0x01,                   // bNumDescriptors
                    0x22,                   // bDescriptorType
                    descr_len as u8,        // wDescriptorLength
                    (descr_len >> 8) as u8, // wDescriptorLength
                ],
            )?;

            writer.endpoint(&self.report_ep)?;

            Ok(())
        }

        fn control_in(&mut self, xfer: ControlIn<B>) {
            let req = xfer.request();

            if req.request_type == control::RequestType::Standard {
                match (req.recipient, req.request) {
                    (control::Recipient::Interface, control::Request::GET_DESCRIPTOR) => {
                        let (dtype, _index) = req.descriptor_type_index();
                        if dtype == 0x21 {
                            // HID descriptor
                            cortex_m::asm::bkpt();
                            let descr_len: u16 = REPORT_DESCR.len() as u16;

                            // HID descriptor
                            let descr = &[
                                0x09,                   // length
                                0x21,                   // descriptor type
                                0x01,                   // bcdHID
                                0x01,                   // bcdHID
                                0x00,                   // bCountryCode
                                0x01,                   // bNumDescriptors
                                0x22,                   // bDescriptorType
                                descr_len as u8,        // wDescriptorLength
                                (descr_len >> 8) as u8, // wDescriptorLength
                            ];

                            xfer.accept_with(descr).ok();
                            return;
                        } else if dtype == 0x22 {
                            // Report descriptor
                            xfer.accept_with(REPORT_DESCR).ok();
                            return;
                        }
                    }
                    _ => {
                        return;
                    }
                };
            }

            if !(req.request_type == control::RequestType::Class
                && req.recipient == control::Recipient::Interface
                && req.index == u8::from(self.report_if) as u16)
            {
                return;
            }

            match req.request {
                REQ_GET_REPORT => {
                    // USB host requests for report
                    // I'm not sure what should we do here, so just send empty report
                    xfer.accept_with(&report(0, 0)).ok();
                }
                _ => {
                    xfer.reject().ok();
                }
            }
        }

        fn control_out(&mut self, xfer: ControlOut<B>) {
            let req = xfer.request();

            if !(req.request_type == control::RequestType::Class
                && req.recipient == control::Recipient::Interface
                && req.index == u8::from(self.report_if) as u16)
            {
                return;
            }

            xfer.reject().ok();
        }
    }
}

use hid::HIDClass;

type LED = gpio::gpioa::PA5<gpio::Output<gpio::PushPull>>;

const PERIOD: u32 = 8_000_000;

#[rtic::app(device = stm32f4xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        counter: u8,
        led: LED,

        usb_dev: UsbDevice<'static, UsbBusType>,
        hid: HIDClass<'static, UsbBusType>,
    }

    #[init(schedule = [on_tick])]
    fn init(mut cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;
        static mut EP_MEMORY: [u32; 1024] = [0; 1024];
        cx.core.DCB.enable_trace();
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            // .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze();

        // assert!(clocks.usbclk_valid());

        let gpioa = cx.device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output();
        usb_dp.set_low().ok();
        delay(clocks.sysclk().0 / 100);
        let usb_dp = usb_dp.into_floating_input();

        let usb_dm = gpioa.pa11;

        let usb = USB {
            usb_global: cx.device.OTG_FS_GLOBAL,
            usb_device: cx.device.OTG_FS_DEVICE,
            usb_pwrclk: cx.device.OTG_FS_PWRCLK,
            pin_dm: usb_dm.into_alternate_af10(),
            pin_dp: usb_dp.into_alternate_af10(),
        };

        *USB_BUS = Some(UsbBus::new(usb, EP_MEMORY));

        let hid = HIDClass::new(USB_BUS.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0xc410, 0x0000))
            .manufacturer("Fake company")
            .product("mouse")
            .serial_number("TEST")
            .device_class(0)
            .build();

        cx.schedule.on_tick(cx.start + PERIOD.cycles()).ok();

        init::LateResources {
            counter: 0,
            led,

            usb_dev,
            hid,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        // rprintln!("idle");
        loop {
            continue;
        }
    }

    #[task(schedule = [on_tick], resources = [counter, led, hid])]
    fn on_tick(mut cx: on_tick::Context) {
        cx.schedule.on_tick(Instant::now() + PERIOD.cycles()).ok();

        let counter: &mut u8 = &mut cx.resources.counter;
        let led = &mut cx.resources.led;
        let hid = &mut cx.resources.hid;

        const P: u8 = 2;
        *counter = (*counter + 1) % P;

        // move mouse cursor horizontally (x-axis) while blinking LED
        if *counter < P / 2 {
            led.set_high().ok();
            hid.write(&hid::report(10, 0));
        } else {
            led.set_low().ok();
            hid.write(&hid::report(-10, 0));
        }
    }

    #[task(binds=OTG_FS, resources = [counter, led, usb_dev, hid])]
    fn usb_fs(mut cx: usb_fs::Context) {
        usb_poll(
            &mut cx.resources.counter,
            &mut cx.resources.led,
            &mut cx.resources.usb_dev,
            &mut cx.resources.hid,
        );
    }

    extern "C" {
        fn EXTI0();
    }
};

fn usb_poll<B: bus::UsbBus>(
    _counter: &mut u8,
    _led: &mut LED,
    usb_dev: &mut UsbDevice<'static, B>,
    hid: &mut HIDClass<'static, B>,
) {
    if !usb_dev.poll(&mut [hid]) {
        return;
    }
}

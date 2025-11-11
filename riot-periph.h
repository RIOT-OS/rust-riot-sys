#ifdef MODULE_PERIPH_ADC
#include <periph/adc.h>
#endif
#ifdef MODULE_PERIPH_CAN
#include <periph/can.h>
#endif
#ifdef MODULE_PERIPH_CPUID
#include <periph/cpuid.h>
#endif
#ifdef MODULE_PERIPH_DAC
#include <periph/dac.h>
#endif
#ifdef MODULE_PERIPH_EEPROM
#include <periph/eeprom.h>
#endif
#ifdef MODULE_PERIPH_FLASHPAGE
#include <periph/flashpage.h>
#endif
#ifdef MODULE_PERIPH_GPIO
#include <periph/gpio.h>
#include <periph/gpio_util.h>
#endif
#ifdef MODULE_PERIPH_HWRNG
#include <periph/hwrng.h>
#endif
#ifdef MODULE_PERIPH_I2C
#include <periph/i2c.h>
#endif
#ifdef MODULE_PERIPH_INIT
#include <periph/init.h>
#endif
#ifdef MODULE_PERIPH_PM
#include <periph/pm.h>
#endif
#ifdef MODULE_PERIPH_PWM
#include <periph/pwm.h>
#endif
#ifdef MODULE_PERIPH_QDEC
#include <periph/qdec.h>
#endif
#ifdef MODULE_PERIPH_RTC
#include <periph/rtc.h>
#endif
#ifdef MODULE_PERIPH_RTT
#include <periph/rtt.h>
#endif
#ifdef MODULE_PERIPH_SPI
#include <periph/spi.h>
#endif
#ifdef MODULE_PERIPH_TIMER
#include <periph/timer.h>
#endif
#ifdef MODULE_PERIPH_UART
#include <periph/uart.h>
#endif
// Disabled as it'd trigger the USB_H_USER_IS_RIOT_INTERNAL checks.
//
// The right way to enable it would be to add a Rust feature to riot-sys,
// off-by-default, that an application pulls in (probably via an equivalent
// flag in riot-wrappers) that enables access to usbdev. Once anything in the
// dependency tree does that, that pulls the whole build into the "needs a
// declared USB ID, and if it's only testing" territory.
//
// #ifdef MODULE_PERIPH_USBDEV
// #include <periph/usbdev.h>
// #endif
#ifdef MODULE_PERIPH_WDT
#include <periph/wdt.h>
#endif

#include <thread.h>
#include <irq.h>
#include <stdio_base.h>

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
#ifdef MODULE_SHELL
#include <shell.h>
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

#include <net/gnrc.h>
#include <net/gnrc/udp.h>
#include <net/gnrc/pktbuf.h>
#include <net/gnrc/ipv6.h>
#include <net/gnrc/nettype.h>
#include <net/gnrc/netapi.h>
#ifdef MODULE_SOCK
#include <net/sock.h>
#endif
#ifdef MODULE_SOCK_UDP
#include <net/sock/udp.h>
#endif
#ifdef MODULE_GCOAP
#include <net/gcoap.h>
#endif
#include <saul.h>
#include <saul_reg.h>
#ifdef MODULE_PTHREAD
// for rwlock
#include <pthread.h>
#endif
#include <board.h>
#ifdef MODULE_XTIMER
#include <xtimer.h>
#endif
#ifdef MODULE_ZTIMER
#include <ztimer.h>
#endif
#include <mutex.h>
#ifdef MODULE_CORD_COMMON
#include <net/cord/common.h>
#endif
#ifdef MODULE_CORD_EP
#include <net/cord/ep.h>
#endif
#ifdef MODULE_CORD_EP_STANDALONE
#include <net/cord/ep_standalone.h>
#endif
#ifdef MODULE_CORD_EPSIM
#include <net/cord/epsim.h>
#endif
#ifdef MODULE_SOCK_UTIL
#include <net/sock/util.h>
#endif

#ifdef MODULE_NIMBLE_HOST
#  include "host/ble_gatt.h"
#  ifndef IS_C2RUST
#    include "host/ble_gap.h"
#  endif
#  include "host/ble_hs_adv.h"
#endif
#ifdef MODULE_NIMBLE_SVC_GAP
#include "services/gap/ble_svc_gap.h"
#endif
#ifdef MODULE_BLUETIL_AD
#include <net/bluetil/ad.h>
#endif

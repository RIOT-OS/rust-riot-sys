#include <shell.h>
#include <thread.h>
#include <irq.h>
#include <stdio_base.h>
#include <periph/adc.h>
#include <periph/gpio.h>
#include <periph/i2c.h>
#include <net/gnrc.h> // C2Rust: needs atomic hack
#include <net/gnrc/udp.h> // C2Rust: needs atomic hack
#include <net/gnrc/pktbuf.h>
#include <net/gnrc/ipv6.h> // C2Rust: needs atomic hack
#include <net/gnrc/nettype.h>
#include <net/gnrc/netapi.h>
#include <net/sock.h>
#include <net/sock/udp.h>
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
#include <xtimer.h> // C2Rust: needs atomic hack
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
#ifdef MODULE_SOCK_UTIL
#include <net/sock/util.h>
#endif

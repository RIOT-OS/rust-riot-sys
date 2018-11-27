#include <shell.h>
#include <thread.h>
#include <stdio_base.h>
#include <periph/adc.h>
#include <periph/i2c.h>
#include <net/gnrc.h>
#include <net/gnrc/udp.h>
#include <net/gnrc/pktbuf.h>
#include <net/gnrc/ipv6.h>
#include <net/gnrc/nettype.h>
#include <net/gnrc/netapi.h>
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

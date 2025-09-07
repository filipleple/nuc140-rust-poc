// bsp/bsp_init.c
#include "NUC1xx.h"
#include "SYS.h"
#include "GPIO.h"
#include "LCD.h"

void bsp_init(void) {
    UNLOCKREG();
    DrvSYS_Open(48000000);   // PLL to 48 MHz
    LOCKREG();

    init_LCD();
    clear_LCD();
}

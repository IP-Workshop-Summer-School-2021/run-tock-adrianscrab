#include "dots_display.h"

#include "tock.h"

bool display_digit(char digit) {
    syscall_return_t res = command(
        0xa0001,
        1,
        digit,
        0
    );
    return res.type == TOCK_SYSCALL_SUCCESS;
}

bool counter_digit(int btn_num) {
    syscall_return_t res = command(
        0xa0001,
        2,
        btn_num,
        0
    );
    return res.type == TOCK_SYSCALL_SUCCESS;
}
#pragma once

#include "tock.h"

static int DRIVER_NUM = 0xa0001;

bool display_digit(char digit);

bool counter_digit(int btn_num);

bool display_text(const char* text);

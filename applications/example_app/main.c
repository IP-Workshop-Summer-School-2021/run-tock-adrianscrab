/* vim: set sw=2 expandtab tw=80: */

#include <stdio.h>
#include <button.h>
// #include "example_driver.h"
#include "dots_display.h"



static void button_callback(int btn_num,
                            int val,
                            __attribute__ ((unused)) int arg2,
                            __attribute__ ((unused)) void *ud) {
  if (val == 1) {
    counter_digit(btn_num);
  }
}

int main(void) {
  // printf ("Hello World!\r\n");
  // example_driver_action ();
  // display_digit(display_digit);

  int err;

  // subscribe la butoane cu functia `button_callback` care 
  // va afisa pe matricea de led-uri
  err = button_subscribe(button_callback, NULL);
  if (err < 0) return err;

  int count;
  err = button_count(&count);
  if (err < 0) return err;

  // activam intreruperi pe cele 2 butoane de pe placa
  // ca la fiecare apasare sa se apeleze `button_callback`
  for (int i = 0; i < count; i++) {
    button_enable_interrupt(i);
  }
  
  return 0;
}

This repository contains some of the arduino exercises of the [Robotic 974](https://www.facebook.com/robotic974) robotics club done in rust

I did this to start learning rust hence it may not be completely idiomatic. 
I tried to achieve these goals:
- device-independent: the exercises are implemented using the [embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/) abstractions and then run on the arduino uno using the [avr-hal](https://github.com/Rahix/avr-hal) implementation of these abstractions. It should be possible to easily run the exercises on something else using the appropriate implementations of the hal (e.g. [esp-hal](https://github.com/esp-rs/esp-hal))
  I split the exercises this way in order to be able to unit-tests them on my computer (it is easier to run tests on a computer than it is to debug an embedded system)
- non-blocking (no delay, no blocking on adc reads) but the debug serial writes might still be blocking.
- documented: I wrote some comments to explain the exercises. Not sure I did enough.

## exercises
See how to run them on your arduino [here](crates/uno/README.md)

- chenillard: 8 leds blinking in a cycle, with a single led turned on at a time
- cowboy: a speed game where you have to press your button faster than your opponents
- debounce: an extra exercice with a software button debounce example. It is done using an exponential moving average on fixed point arithmetic.
- vumetre: turn on part of a led array after a potentiometer position.
- dice: a virtual dice that is cast on the press of a button. The result is displayed as a number of turned on leds.

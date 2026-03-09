# GPIO: General-Purpose I/O

**GPIO** (General-Purpose Input/Output) pins are the simplest form of digital hardware interface. Each pin can be configured as either an input (reading a digital HIGH or LOW value) or an output (driving HIGH or LOW).

## Use Cases

- Reading push buttons or switches
- Driving LEDs or relays
- Digital sensor inputs (limit switches, proximity sensors)
- Control signals to external circuits

## Configuration

```toml
[[tasks]]
type = "Gpio"
name = "DigitalIO"
pins = [1, 2, 3, 4]
```

The `pins` array lists the GPIO pin numbers to control. In r-sim's simulated mode, each pin toggles between 0.0 and 1.0.

## I/O Lifecycle

The GPIO task follows the full I/O lifecycle:

```
initialize_io()  →  [once at startup]
   ↓
read_inputs()    →  [every time step - sample pin states]
execute()        →  [compute logic based on pin states]
write_outputs()  →  [drive output pins]
```

## Example Output

```
GPIO Task DigitalIO initialized with pins: [1, 2, 3, 4]
Initializing I/O for task: DigitalIO
Reading I/O for task: DigitalIO
  GPIO Task DigitalIO reading from pin 1
  GPIO Task DigitalIO reading from pin 2
Executing GPIO Task DigitalIO: 500ms
Writing I/O for task: DigitalIO
  GPIO Task DigitalIO writing to pin 1
```

## Connecting to Real Hardware

In production, the simulated GPIO reads/writes would be replaced with calls to the system's GPIO subsystem (e.g., `/sys/class/gpio` on Linux, or a HAL library). The task interface remains the same — only the implementation changes.

> **Exercise:** Create a configuration with two GPIO tasks: one reading pins [1, 2] and one writing to pins [3, 4]. Connect them with a dependency so that read values flow from the reader to the writer.

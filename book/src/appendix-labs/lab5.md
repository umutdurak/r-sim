# Lab 5: I/O and Hardware Interfacing

## Objective
Configure and test multiple I/O task types. Understand the I/O lifecycle (initialize → read → execute → write) and synchronized multi-I/O execution.

## Background
Review: [Chapter 5](../ch05-io.md)

## Steps

### Part A: GPIO (30 min)

1. Create `lab5_gpio.toml`:
   ```toml
   [[tasks]]
   type = "Gpio"
   name = "DigitalSensors"
   pins = [1, 2, 3]
   ```
2. Run and observe: init, read, execute, write phases

### Part B: Serial (30 min)

1. Create `lab5_serial.toml`:
   ```toml
   [[tasks]]
   type = "Serial"
   name = "RS232_Sensor"
   port = "/dev/ttyUSB0"
   baud_rate = 115200
   ```
2. Run and compare output with GPIO task

### Part C: UDP (30 min)

1. Create `lab5_udp.toml`:
   ```toml
   [[tasks]]
   type = "Udp"
   name = "NetworkLink"
   local_addr = "127.0.0.1:18080"
   remote_addr = "127.0.0.1:18081"
   ```
2. Run and verify UDP binding

### Part D: Synchronized Multi-I/O (45 min)

1. Create `lab5_multi_io.toml` with GPIO + UDP + Analog:
   ```toml
   [[tasks]]
   type = "Gpio"
   name = "GPIO_Sensors"
   pins = [1, 2]

   [[tasks]]
   type = "Udp"
   name = "UDP_Link"
   local_addr = "127.0.0.1:19090"
   remote_addr = "127.0.0.1:19091"

   [[tasks]]
   type = "Analog"
   name = "ADC_Input"
   channels = [0, 1, 2, 3]
   is_input = true
   sampling_rate_hz = 1000
   ```
2. Run and verify that ALL I/O tasks are initialized and read/written each step

## Deliverables

1. Console output for each I/O type showing the init→read→execute→write cycle
2. Table comparing the I/O lifecycle across GPIO, Serial, and UDP (which methods produce output? what do they print?)
3. Configuration file for the multi-I/O test

## Challenge

Design a configuration simulating a complete embedded system: GPIO for switches, Analog for temperature sensors, Serial for a display, and Modbus for a PLC. How would you connect these with dependencies?

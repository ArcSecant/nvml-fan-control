# nvml-fan-control
headless gpu fan control on linux

# Usage
Build the binary with `cargo build --release` and run with `sudo target/release/nvml fan_speed`.
The fan curve is configurable via the `fan_speed` file (you can also specify a different file).
The first number in each line represents the temperature and the second is the fan speed. For example:

```
30 20
40 30
50 60
```
means that
```
temp   speed
<30    20%
30-40  30%
40-50  60%
>50    60%
```

# NUC140 “Hello from Rust!” proof of concept

Minimal hybrid **Rust + C BSP** setup for the Nuvoton **NUC140 (Cortex-M0)** that prints `Hello from Rust!` on the LCD using your existing vendor libraries.

---

## Prereqs

* `arm-none-eabi-gcc`, `arm-none-eabi-objcopy`, `arm-none-eabi-size`
* Rust + target: `rustup target add thumbv6m-none-eabi`
* Your vendor `Library/` tree present (NUC1xx, LCD, SYS, GPIO, CMSIS, etc.)

### (Optional) Docker

Use the image you built earlier:

```bash
docker build -t nuc140-rust .
docker run --rm -it -v "$PWD:/workdir" -w /workdir nuc140-rust bash
```

---

## Layout

```
nuc140-rust-hello/
  memory.x                     # flash/ram sizes
  .cargo/config.toml           # linker choice (see below)
  build.rs                     # compiles BSP C → libbsp.a
  src/main.rs                  # calls LCD print_Line()
  bsp/bsp_init.c               # board init (UNLOCKREG, PLL, LCD init)
  Library/...                  # your vendor sources/headers
```

---

## Linker choice (pick ONE)

### A) `rust-lld` + libgcc (simple & fast)

`.cargo/config.toml`

```toml
[build]
target = "thumbv6m-none-eabi"

[target.thumbv6m-none-eabi]
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Map=target/thumbv6m-none-eabi/release/nuc140.map",
]
```

`build.rs` (after `b.compile("bsp")`) adds libgcc:

```rust
let gcc = b.get_compiler();
let libgcc = std::process::Command::new(gcc.path())
    .arg("-print-libgcc-file-name").output().unwrap();
println!("cargo:rustc-link-arg={}", String::from_utf8(libgcc.stdout).unwrap().trim());
```

### B) GCC as the linker (auto pulls libgcc/newlib)

`.cargo/config.toml`

```toml
[build]
target = "thumbv6m-none-eabi"

[target.thumbv6m-none-eabi]
linker = "arm-none-eabi-gcc"
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Wl,-Map=target/thumbv6m-none-eabi/release/nuc140.map",
  "-C", "link-arg=-Wl,--gc-sections",
  "-C", "link-arg=-Wl,--nmagic",
]
```

---

## Key files

`memory.x`

```ld
MEMORY { FLASH : ORIGIN = 0x00000000, LENGTH = 128K
         RAM   : ORIGIN = 0x20000000, LENGTH = 16K }
```

`bsp/bsp_init.c`

```c
#include "NUC1xx.h"
#include "SYS.h"
#include "GPIO.h"
#include "LCD.h"

void bsp_init(void) {
    UNLOCKREG();
    DrvSYS_Open(48000000);
    LOCKREG();
    init_LCD();
    clear_LCD();
}
```

`src/main.rs`

```rust
#![no_std]
#![no_main]

use core::ffi::c_char;
use cortex_m_rt::entry;
use panic_halt as _;

extern "C" { fn bsp_init(); fn print_Line(line: i32, s: *const c_char); }

#[entry]
fn main() -> ! {
    unsafe { bsp_init(); }
    static HELLO: &[u8] = b"Hello from Rust!\0";
    unsafe { print_Line(0, HELLO.as_ptr() as *const c_char); }
    loop { cortex_m::asm::nop(); }
}
```

`build.rs` (C BSP compile — **no** startup files!)

```rust
let mut b = cc::Build::new();
b.target("thumbv6m-none-eabi")
 .compiler("arm-none-eabi-gcc")
 .flag("-mcpu=cortex-m0").flag("-mthumb")
 .flag("-ffunction-sections").flag("-fdata-sections")
 .flag("-fno-builtin").flag("-fno-common").flag("-Os")
 .define("__EVAL", None).define("__UVISION_VERSION", Some("542"))
 .include("Library/CMSIS/Include")
 .include("Library/Device/Nuvoton/NUC1xx/Include")
 .include("Library/Device/Nuvoton/NUC1xx/Source")
 .include("Library/NUC1xx/Include")
 .include("Library/NUC1xx-LB_002/Include")
 .file("Library/NUC1xx/Source/GPIO.c")
 .file("Library/NUC1xx/Source/SYS.c")
 .file("Library/NUC1xx-LB_002/Source/LCD.c")
 .file("Library/Device/Nuvoton/NUC1xx/Source/system_NUC1xx.c")
 .file("bsp/bsp_init.c");
b.compile("bsp");
```

---

## Build

```bash
cargo build --release

# Make a .bin from the THUMB ELF
arm-none-eabi-objcopy -O binary \
  target/thumbv6m-none-eabi/release/nuc140-rust-hello \
  nuc140-rust-hello.bin
```

---

## Flash

* Program **APROM @ 0x0000\_0000** (mass-erase once), then hardware reset.
* Use Nu-Link ICP (point at the ELF or BIN) or OpenOCD (`program ... verify reset exit`).

---

## Sanity checks (do these before flashing)

```bash
# Non-zero sizes
arm-none-eabi-size --format=berkeley target/thumbv6m-none-eabi/release/nuc140-rust-hello

# Vector table at 0x00000000
arm-none-eabi-objdump -h target/thumbv6m-none-eabi/release/nuc140-rust-hello | \
  egrep -i 'vector|isr|\.text|\.data|\.bss'

# Exactly one Reset (from cortex-m-rt)
arm-none-eabi-nm -n target/thumbv6m-none-eabi/release/nuc140-rust-hello | \
  egrep -i ' Reset$|DefaultHandler|HardFault'
```

---

## Troubleshooting

* **0-byte `.bin` / all zeros from `size`:** you linked against the wrong script or GC’ed everything. Use `link.x` + `memory.x` as above.
* **`undefined symbol: __gnu_thumb1_case_*`:** link `libgcc` (Option A’s `-print-libgcc-file-name`) or use GCC as linker (Option B).
* **`undefined reference to bsp_init`:** make sure `bsp_init.c` is compiled, not `static`, and shows up in `libbsp.a` (`arm-none-eabi-nm libbsp.a | grep bsp_init`).
* **Old code still runs / weird mix:** flash APROM (not LDROM), mass erase once, reset after program.

---

## Extending

Add more BSP C files in `build.rs` (`Scankey.c`, `SPI.c`, `ff.c`, `diskio.c`, etc.), then declare the functions in Rust with `extern "C"` using `core::ffi` types. Avoid calling C **variadics** from Rust; add tiny C shims with fixed arguments instead.

Have fun! 🚀


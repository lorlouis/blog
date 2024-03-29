---
Title: You don't have to boot from just 512 bytes
Author: Louis
Date: 2023-05-28
Blurb: As long as you boot from a CD
---
# You don't have to boot from just 512 bytes

## Wait, what?

Conventional wisdom says that you can only boot from the first sector of a
floppy (512 bytes) or something that looks and behaves like the first sector of
a floppy. But it doesn't have to be the case as long as you boot from a CD.

### The "normal" booting process

Historically the IBM PC did not ship with a hard drive; it had a BASIC
interpreter in its ROM and up to 2 floppy disk drives. If you wanted a proper
operating system, the PC had to boot from a floppy containing an OS. The BIOS
looked for the magic numbers `[0x55, 0xAA]` at the end of each floppy's first
segment to detect if it could boot from it. Once a bootable drive was found,
the segment was loaded into memory at address `0x7C00`, and the CPU started
executing at that address. When hard drives came along, a similar technique was
used to boot from the [MBR](https://en.wikipedia.org/wiki/Master_boot_record),
but only 446 byes were available[^1] compared to the floppies' 510.

Unsurprisingly, most modern PCs still support this booting mechanism. However,
some manufacturers have started to remove support for legacy BIOS booting in
favour of UEFI, but that's a story/rant for another time.

## A Minimal Bootable ISO

### A tiny bit of context

An ISO file is *just* a file containing an ISO 9660 file system which is the
file system that CDs use. PCs they boot off CDs via the `El Torito`[^2]
extension to ISO 9660 standard.

The format is pretty straight forwards:

```no-hi
An ISO 9660 with EL TORITO extension
  (the bits to boot a PC at least)
  Offset
  0x0000_ _____________
         |    ....     |
         |  <unused>   |
         |    ....     |
  0x8000_|_____________|
  0x8800_|_primary_vol_|
         |_boot_record_| --.
         |    ....     |    |
         <other volumes>    |  addr
         |    ....     |    | of boot
         |_____________|    | catalog
         |__terminator_|    |
     .-- |_boot_catalog| <-´
     `-> |__boot_image_|
         |    ....     |
         |<rest of the |
         | file system>|
         |    ....     |
          ¯¯¯¯¯¯¯¯¯¯¯¯¯
```

* The first 0x8000 bytes are unused, go wild and use them however you want.
  These bytes were left unused by the specification to allow for other booting
  systems to work on CDs. When "burning" an ISO to a thumb drive, this section
  generally contains an MBR or the UEFI equivalent.

* The CD is segmented into fixed-sized segments, in most cases, 2048 bytes each.

* The first segment used is at offset 0x8000, called the `Primary Volume Descriptor`.

* The second segment at offset 0x8800 *may* be a `Boot Record`.

* It is not required to read the CD's filesystem to boot from a CD.

That last point piqued my interest. If you only care about finding something
that looks like a floppy and boot it, you can ignore most of the filesystem.
I wanted to see just how little of the spec I had to implement to build a
 `Minimal Bootable ISO`.

### El Torito basics

El Torito defines two sections of the CD, the `Booting Catalog`, which is
comprised of multiple entries containing information about one or more bootable
payloads. And the `Boot Record Volume`, which the BIOS uses to find the boot
catalog.

#### Boot Record Volume Descriptor

````no-hi
          Boot Record Volume Descriptor
 _______________________________________________
|Offset_|__type___|____________Desc_____________|
|_0x000_|___u8____|__boot_record_indicator_=_0__|
| 0x001 |         |                             |
|  ...  | [u8; 5] | ISO-9660 identifier ="CD001"|
|_0x005_|_________|_____________________________|
|_0x006_|___u8____|_________version_=_1_________|
| 0x007 |         |   Boot system identifier    |
|  ...  | [u8;32] | ="EL TORITO SPECIFICATION"  |
|_0x026_|_________|_____________________________|
| 0x027 |         |                             |
|  ...  | [u8;32] |     Unused, "must" be 0     |
|_0x046_|_________|_____________________________|
| 0x047 |         |Sector id of the boot catalog|
|  ...  |   u32   |   sec_id * 2048 = offset    |
|_0x04a_|_________|_____________________________|
| 0x04b |         |                             |
|  ...  |[u8;1977]|     Unused, "must" be 0     |
| 0x7ff |         |                             |
 ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
    * All multi byte numbers are in little endian
````

The only value that changes is the `sector id of the boot catalog` (bytes `0x47`
to `0x4a`). Everything else is either zeroed or a magic value of some sort.

#### The Boot Catalog

The boot catalog defines where the boot payload(s) are located.
And is stored across one or more segments and is composed of a
series of entries.

```no-hi
                    The Boot Catalog
bytes 0x00 ........ 0x1f
 0x00 [Validation Entry] <- makes sure the data is not corrupted
 0x20 [  Initial Entry ] <- contains info about a boot payload
 0x40 [ Section Header ] <- info about section entries (optional)
 0x60 [ Section Entry 1] <- info about a boot image 1 (optional)
 0x80 [  Entry Ext 1   ] <- 13 bytes of data* (optional)
  --  |       :        |
 0x?? [ Section Entry N]
 0x?? [   Enty Ext N   ]

    * Multiple `Entry Ext` can be chained together.
```

For an ISO containing only one boot payload, we only need to consider the
`Validation Entry` and the `Initial Entry`.

### The Validation Entry

The validation entry is used to detect if the content is corrupted.

```no-hi
              Validation Entry
 ______________________________________________
|Offset|__type___|____________Desc_____________|
|_0x00_|___u8____|________header_id_=_1________|
|_0x01_|___u8____|____platform_id_=(1|2|3)_____|
| 0x02 |   u16   |     Unused, "must" be 0     |
|_0x03_|_________|_____________________________|
| 0x04 |         |                             |
|  ..  | [u8;24] |      manufacturer id        |
|_0x1b_|_________|_____________________________|
| 0x1c |   u16   |      checksum reserved      |
|_0x1d_|_________|_____________________________|
|_0x1e_|___u8____|____________0x55_____________|
| 0x1f |   u8    |            0xaa             |
 ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
```

`Platform id` is interesting because it was originally defined as:

```rust
#[repr(u8)]
enum PlatformId {
    x86     = 0x0,
    PowerPC = 0x1,
    Mac     = 0x2,
}
```

But Mac, in this case, the Mac platform pre-Intel, never implemented booting
off a CD using El Torito. Although not in the standard, `0xef` is commonly used
to identify bootable images that rely on UEFI.

This is the enum I ended up using:

```rust
#[repr(u8)]
pub enum Platform {
    X86 = 0,
    PPC = 1,
    Mac = 2, // mac is never used ?
    UEFI = 0xef, // not part of the spec..
}
```

The other noteworthy field is `checksum reserved`. A checksum is computed by
summing up the whole segment as a list of `u16`. This reserved `u16` is used to
ensure the sum wraps around to zero.

### The Initial Entry

The second entry in the catalog is the initial entry; it contains info on a
segment containing a bare metal 16-bit "real mode" executable and how to load
it into memory.

```no-hi
                Initial Entry
 ______________________________________________
|Offset|__type___|____________Desc_____________|
|_0x00_|___u8____|_boot_indicator_=(0x88|0x00)_|
|_0x01_|___u8____|___boot_media_type_=(0..=4)__|
| 0x02 |   u16   |      Load Segment addr      |
|_0x03_|_________|_____________________________|
|_0x04_|___u8____|_________system_type_________|
|_0x05_|___u8____|_____Unused_"must"_be_0______|
| 0x06 |   u16   |       Sector Count          |
|_0x07_|_________|_____________________________|
| 0x08 |         |    Block address of the     |
|  ..  |   u32   |         bootloader          |
|_0x0b_|_________|_____________________________|
| 0x0c |         |                             |
|  ..  | [u8;17] |     Unused "must" be 0      |
| 0x1f |         |                             |
 ¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯¯
```

A  boot indicator of value `0x88` marks the entry as bootable, which, in
practice, is almost always set. The boot media type lets the BIOS expose this
sector to the executable as if it were a floppy, a hard drive or a CD. This
lets older operating systems like DOS boot and read data from a CD as if it
were a floppy without needing any extra drivers.

#### Sector count

`Sector count` tells the BIOS how many sectors of the emulated device it should
load into memory. This lets you load more than one floppy segment. In CD mode,
this would let you load up to `128MB` of data into memory, crushing the merger
`510B` (if that) level 1 bootloaders need to restrict themselves to.

## Booting a payload larger than 512 bytes

I uploaded the code I used to build my `Minimal Bootable ISO` to GitHub under
<https://github.com/lorlouis/iso9660>. Calling `make` will create a disk image
and run it via QEMU.
[`src/bin/bootable.rs`](https://github.com/lorlouis/iso9660/blob/main/src/bin/bootable.rs)
contains the steps to create the ISO. The steps loosely resemble:

1. Create a primary header

```rust
let primary_header = VD {
    ty: VDType::PrimaryVD,
    version: 1,
};
```
This is needed as [SeaBIOS](https://www.seabios.org/SeaBIOS), the default i386
BIOS implementation in QEMU, checks to see if what's in the CD drive *really*
is a CD.

2. Create a boot record of the El Torito variety

```rust
let boot_record = BootRecord::el_torito(18);
```
`18` here denotes the sector 18 at which the boot catalog will be placed. The
first 15 sectors are unused, the primary volume descriptor uses the 16th, and
the 17th is the boot record, which leaves the 18th sector free.

3. Create a validation entry

```rust
let validation = ValidationEntry {
    header_id: 1,
    platform_id: Platform::X86,
    manufacturer_id: None,
};
```

SeaBIOS does not check the sector's checksum so the `checksum reserved`
field is filled with 0s.

4. Create the initial entry

```rust
let initial = InitialEntry {
    boot_indicator: BootIndicator::Bootable,
    boot_media: BootMedia::Floppy1_44,
    load_segment: 0, // ie default value (I know it should be an option)
    sys_type: 0,  // no idea what it's supposed to be, idk it felt right
    sector_count: 4, // hmm intresting
    virtual_disk_addr: 19, // the last segment
};
```

`sector_count` is set to 4 because of the boot media emulation. Floppy sectors
are 512 bytes long, and a CD sector is 2048 bytes long. It is possible to load
more than that, but I did not see any need for this proof of concept.

5. The last step is to concatenate the files into an ISO

```make
# create the 20 sectors required
dd if=/dev/zero of=$(ISO_FILE) count=20 bs=2048
# copy iso data in sector 17 and 18
dd if=$(ISO_DATA) of=$(ISO_FILE) seek=16 count=3 bs=2048 conv=notrunc
# copy stage 1
dd if=$(STAGE1_BIN) of=$(ISO_FILE) seek=$((19*4)) count=4 bs=512 conv=notrunc
```

The executable I loaded in the last sector was generated from this assembly
```x86asm
org 0x7c00 ; address at which the bios will load this executable
bits 16 ; 16 bit mode

    ; initialise pointers
    mov ax, 0
    mov ds, ax ; data segment 0
    mov ss, ax ; stack segment 0
    mov es, ax ; extra segment 0?
    mov sp, 0x7c00 ; set stack pointer at the start of this executable

_start:
    mov si, hello
    call puts
    jmp other ; jump into code after the 512th byte

; si=str, cl=strlen
puts:
    lodsb
    or al, al
    jz .done
    call putc
    jmp puts
.done:
    ret

; al=char
putc:
    mov ah, 0eh
    int 10h
    ret

hello: db 'hello world!', 10, 13, 0
hello_len: equ $-hello

meme: db 'hello meme!', 0
meme_len: equ $-meme

times 510 - ($ - $$) db 0 ; fill with 0s until bytes 511
db 0x55, 0xaa ; mark the sector as bootable by setting the bytes 511 and 512

other:
    mov si, meme
    call puts
    hlt

times 2048 - ($ - $$) db 0 ; fill the rest of the disk sector with 0s
```

## Is this even remotely useful?

**No.**

Most of the questions asking how to boot off more than 512 bytes come from
people trying to avoid writing a multi-stage bootloader, even though there are
many benefits to separating your bootloader in stages. This article details a
quirk of booting off a CD on the PC platform. None of it applies to ISOs burnt
to USB drives or booting from a hard drive and thus would still require you to
implement multi-stage booting.

[^1]: <https://en.wikipedia.org/wiki/Master_boot_record#Sector_layout>
[^2]: <https://pdos.csail.mit.edu/6.828/2014/readings/boot-cdrom.pdf>

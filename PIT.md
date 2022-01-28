# PIT file format documentation

This is the first attempt (that I know of) to publically document the Samsung partition information file (PIT) format.

Most of this info was obtained by reading through the Heimdall source code.

The same disclaimers as in `PROTOCOL.md` apply. This information is provided on a best-effort basis. If this bricks your device, you get to build your house with it.

## Base types

These are similar to the ones used in the Odin protocol:

* `Integer`: 32 bit, little endian, unsigned
* `Short`: 16 bit, little endian, unsigned
* `String`: `NULL`-terminated, C-style string. In ASCII or at least a compatible encoding. All currently known strings are at most 32 bytes total.

## Magic

All known PIT files start with the magic bytes `[0x76, 0x98, 0x34, 0x12]`.

## Headers

The magic is followed by a header describing the file. Most of the entries in this header are not yet known.

The first and most important entry is an Integer stating the number of partition entries that will follow.

This is followed by 2 unknown integers and 6 unknown Shorts.

## Partition entries

Immediately following the table are the actual partition entries. These are 132 bytes each.

The entries are made of the following fields. They are listed in order:

### Binary type (Integer)

* `0x00`: This is a firmware for the Application processor (AP), AKA the "Android/Tizen partition". At least that's what Heimdall says. In practice, it seems to simply mean "this partition is not for the modem".
* `0x01`: This is a firmware for the modem.

### Device type (Integer)

* `0x00` "One NAND": Not quite sure what this means, but this is what it's called in Heimdall.
* `0x01` "File": Same story.
* `0x02` "MMC": Same story.
* `0x03` "All": Same story. In fact, there's even a `// ?` comment in Heimdall source behind this.

### Identifier (Integer)

Not quite sure what this means. It seems to count up in subsequent entries, but sometimes also has bizzare values
(like `70` or `80`, followed by `1`, `2`, `3` etc).

### Attribute (Integer)

Not quite sure here either. This contains C-style flags which can be ORed together.

* `0x01` "Write": Don't know.
* `0x02` "STL": Most likely "Sector Transition Layer"
* `0x04` "BML": Most likely "Block Management Layer"

STL and BML are device interfaces for block devices, the BML interface is used for the rootfs.

### UpdateAttributes (Integer)

Update Attribute flags, possibly to force signature checks or something like that.

* `0x01` "Fota": Probably implies that this partition can receive OTA updates.
* `0x02` "Secure": Probably implies that this partition is locked and can only be flashed with signed images.

### BlockSizeOrOffset (Integer)

This property seem to have multiple usages depending in the bootloader used. Heimdall states that this is either the Block Size or the Offset

In the Watch we're reversing this property is set to a lot of diferent values rising and not multiples of 2, which seems to be an indication that it is used as Offset in our case. You'd probably have to implement a heuristic that guesses this in order to implement it properly.

### BlockCount (Integer)

This property might describe the size of the partitions in blocks

### FileOffset (Integer)

This property is marked Obsolete and is only parsed and written but never interpreted by Heimdall.

It seems that this is always 0 in newer PIT files. However, it may still mean something in older ones.

It'd probably be helpful to analyze the PIT of a Galaxy S or similarly ancient device to clear this up.

### FileSize (Integer)

This property is marked Obsolete and is only parsed and written but never interpreted by Heimdall.

It seems that this is always 0 in newer PIT files. However, it may still mean something in older ones.

It'd probably be helpful to analyze the PIT of a Galaxy S or similarly ancient device to clear this up.

### PartitionName (String, 32 Bytes)

Name of the Partition. Was All Uppercase with optional number on end in our case
Examples:

* MODULE
* STEADY
* WD-RESERVED

### FlashFileName (String, 32 Bytes)

Name of the File. Sometimes there is a `-` Displayed. Maybe when no data is there or it is litarally a `-` Examples:

* sboot.img
* loader.img

### FotaFileName (String, 32 Bytes)

This name was always empty in our case, even when Fota Flag is set.

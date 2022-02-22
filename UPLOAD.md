# Upload mode

Upload mode is a Samsung debug feature that hasn't been talked about much on the Internet. It's main feature is the ability to request memory dumps from the target. It can be entered by key combos on most Samsung devices, and is entered automatically after some kernel panics (though usually only if the debug level is set to high via the magic `*#9900#` dialer code's menu).

## Prior work

Other (incomplete?) implementations of this mode can be found at:

* <https://github.com/nitayart/sboot_dump>
* <https://github.com/bkerler/sboot_dump>

This mode has been somewhat documented in [this blogpost](https://hexdetective.blogspot.com/2017/02/exploiting-android-s-boot-getting.html). However, the details about the probe command were obtained by reading bkerler's implementation. All wireless upload-specific information has been obtained by reverse engineering the Gear S2 classic's and Gear S3's firmwares.

### Initializing the connection

Connection establishment over USB works in the same way as for USB download mode in terms of finding the correct endpoints.

For wireless upload, Samsung for whatever reason decided to reverse the roles of host and target. Rather than the target connecting to port 1359 on the host, instead the host connects to that port on the target (which seems to always have IP 192.168.49.1 on the watches we looked at).

Then the host sends the C string "PrEaMbLe\0" (note the NULL terminator). Like all commands, the packet needs to be 0-padded to 1024 bytes in size.

The target should reply with "AcKnOwLeDgMeNt\0".

### Getting information about memory layout

Send "PrObE\0" to the target. It will respond with a "probe table" data structure. It's laid out as follows:

```C
struct probetable {
 /*
 Name of the device.
 According to bkerler's implementation, if this starts with a +", the device is 32-bit.
 Otherwise, the device is 64-bit.

 TODO: Is this NULL-terminated?
 TODO: Would a separate mode field make more sense?
 */
 char[16] device_name;

 /*
 Entries in the probe table.

 For 32-bit devices, an entry is 28 bytes total.
 For 64-bit devices, it's 40 bytes total.

 The number of entries is dynamic, but the last entry always has it's
 start and end addresses set to 0.
 bkerler's implementation also takes a start address of < 20 as a sign to end.
 Not sure what's up with that.
 */
 struct probetable_entry[] entries;
}

struct probetable_entry {
 // Type of partition. Needs further investigation.
 uint32_t partition_type;
 // Name of partition, NULL-terminated.
 char[12] pname;
 // Some kind of additional information. 64-bit only. Needs further research.
 uint64_t info;
 // Start address of memory area. Size is bitness-dependent.
 uint32_or_64_t start_addr;
 // End address of memory area. Size is bitness-dependent.
 uint32_or_64_t end_addr;
}
```

### Transfering a chunk of memory

Send a packet with the first memory address to be included in the dump as a 32 or 64-bit little-endian uint, 0-padded to 1024 bytes.

Repeat with the last memory address to be included in the dump.

Finally, send a 1024 byte-padded packet with the C string "DaTaXfEr\0".

The target will now reply with a byte stream of the size of the requested memory chunk.

You should acknowledge each transfer by sending a packet with the C string "AcKnOwLeDgMeNt"
after reading the chunk.

According to bkerler's code, the target also (only sometimes?) sends "PoStAmBlE\0" after a transfer.

Note that the maximum chunk size is `0x80000` bytes (512KiB).
If you want to dump more than that you'll have to do multiple transfers.

### Ending the session

Send "PoStAmBlE\0" to the target to end the session. It doesn't seem to send anything back.

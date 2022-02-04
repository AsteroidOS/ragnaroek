# Upload mode

Upload mode is a Samsung debug feature that hasn't been talked about much on the Internet. It's main feature is the ability to request memory dumps from the target. It can be entered by key combos on most Samsung devices, and is entered automatically after some kernel panics.

## Prior work

Incomplete implementations of this mode can be found at

* <https://github.com/bkerler/sboot_dump>
* <https://github.com/bkerler/sboot_dump>

This mode has been somewhat documented in [this blogpost](https://hexdetective.blogspot.com/2017/02/exploiting-android-s-boot-getting.html)

### Initializing the dump

After establishing a connection to the device (from what I can tell, this works just like in download mode but withou the magic "ODIN" -> "LOKE" handshake or session begin packet), send the C string "PrEaMbLe\0" (note the NULL terminator). The packet needs to be 0-padded to 1024 bytes in size.

### Transfering a chunk of memory

Send a packet with the first memory address to be included in the dump as an Odin Integer (? What about 64 bit?), 0-padded to 1024 bytes.

Repeat with the last memory address to be included in the dump.

Finally, send a 1024 byte-padded packet with the C string "DaTaXfEr\0".

The device will now reply with a byte stream of the size of the requested memory chunk.

TODO: Confirm experimentally

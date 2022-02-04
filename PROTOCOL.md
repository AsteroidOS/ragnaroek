# ODIN Protocol documentation

This document seeks to briefly describe the Odin protocol.

A lot of this information was pulled from Heimdall and other open source tools,
but to my knowledge this is the first document that strives to collect this information in a human-readable form.

This document is not (yet) exhaustive, so please contribute if you find out anything yourself. Also, most of this analysis was performed based on NetOdin. Regular USB Odin may differ somewhat.

## Packet format

### Base types

* `Integer`: 32 bit, little endian, unsigned
* `Short`: 16 bit, little endian, unsigned

### Magic packets

These serve to establish consensus between the flasher and target that they are indeed talking to each other. As such, they are exchanged immediately after the target connects.

After the target connects to the flasher, the flasher sends the ASCII string "ODIN" (4 byte packet, no NULL termination).

The target replies with the ASCII string "LOKE" (once again a 4 byte packet, no NULL termination).

### Command packets

All these packets are sent by the flasher to the target in order to command a certain action.

To our knowledge, they are always padded to exactly `1024` bytes, both over USB and the network.

#### Session start packet

This packet is sent to the device after the magic packet exchange, but before any other command.

It's not entirely clear whether the session has to be re-negotiated after every command or only once.
Heimdall does the former, whereas Samsung Odin seems to do the latter.

##### Flasher to target

These packets start with the Integer `0x64`.

This is followed by a flag Integer which may have the following values taken directly from Heimdall sources. Because of the poor documentation of the code it's not yet entirely clear what each of them do, and it's likely that this list is not exhaustive:

* `0x00`: "Begin session". This always seems to be the value in the very first session setup packet.
* `0x01`: "Device type" (based on the Heimdall name). Not yet sure what this means exactly.
* `0x02`: "Total bytes". Same story here.
* `0x03`: "Enable some sort of flag". Even the author of Heimdall seems to not know what this is supposed to mean.
* `0x05`: "File part size". Same story as above.
* `0x08`: "T-Flash mode". Flash an external SD card inserted into the target rather than the target's internal storage.

*TODO: Further bytes*

It's also important to note that this packet may be sent multiple times in a row (after waiting for the target's response). I suspect this is so you can set multiple flags.

##### Target to flasher

The target responds to each packet sent by the flasher with an 8 byte long packet. The first Integer is always `0x64`. The second one appears to be either `0x00`, or `0x0300` (*FIXME Check endianness*) if the second Integer of the flasher's packet was `0x03` (or other circumstances that aren't known yet). This may or may not be safe to ignore.

#### PIT transfer packet

This packet type is used to transfer PIT partition description between flasher and target.

The PIT format itself is documented in `PIT.md`.

##### Flasher to target

The first Integer is always `0x65`. This is followed by an action specifier:

* `0x00` "Request Flash": Asks the target to repartition itself based on the following PIT file
* `0x01` "Request dump": Asks the target to transfer the current partition layout in following packets
* `0x02` "Request part": Tells the target how far the transfer has come. Typically sent from the second packet onwards.
* `0x03` "Request end of file transfer": Tell the target that the transfer is over. Typically sent after the expected amount of data has been transferred.
* `0x05` "Change packet size": According to <https://i.blackhat.com/USA-20/Wednesday/us-20-Chao-Breaking-Samsungs-Root-Of-Trust-Exploiting-Samsung-Secure-Boot.pdf>, this can be used to tell the target the size of data packets to expect. Needs further research/confirmation.

Your first PIT packet should always set this to either `0x00` or `0x01`. Subsequent packets to `0x02` or `0x03`.

If the flag `0x02` has been set in the second Integer, the third Integer must be set to the index of the next 500-byte PIT chunk to be transferred. For example, if you've been dumping and already have received 2000 bytes of PIT data, you'd set this to `0x04`.

##### Target to flasher

If the second flasher packet Integer was `0x01` (dump), the reply to the first packet will be exactly 8 bytes long. The first Integer will always be `0x65`, the second will state how much data to expect (in bytes).

If the second flasher packet Integer was `0x02`, the reply will contain the requested part of the PIT file as binary data. The packet will be exactly 500 bytes large if there's enough data remaining, smaller otherwise.

If the second flasher packet Integer was `0x03`, the reply will look like for `0x01`, except that the second Integer is always `0x00`.

* TODO: Investigate how this looks like for `0x00` (a flash command)*

#### Flash (Odin AP package) packet

These packets are used to actually transfer the firmware for a given partition. The way this works is quite similar to PIT transfers.

##### Flasher to target

The first Integer is always `0x66`. This is followed by an action specifier:

* `0x00` "Request flash": Asks the target to await further instructions for receiving and flashing a partition image.
* `0x01` "Request dump":  Asks the target to start sending partition contents.
* `0x02` "Request chunk": Asks the device to send the next chunk of data. Not sure whether this is relevant while flashing, or only while dumping.
* `0x03` "Request end": Signals to the device that the transfer is over.

The first packet should always be either `0x00` or `0x01`. Subsequent packets `0x02` or `0x03`.

After the first packet, a request flash packet seems to be resent by NetOdin.

Afterwards, the real transfer starts. Packets of 1460 bytes are sent until 131072 bytes (exactly 128KB) have been transferred. The 1460 byte packets are probably just because of the 1500 byte MTU.

After the 128KB chunk has been transferred, the target responds with the Integer 0x00, followed by the chunk index (also an Integer). It is currently not known whether this index may ever reference a previously-sent chunk (thus requesting retransmission).

This is repeated with all chunks until the sender has no more data. Then, a Request End packet is sent to the target to signal end of transmission.

What's done if the partition image is not a multiple of 128KB in size needs to be researched.

These notes are also based on observing NetOdin flashing an AP tarball. How to flash individual partitions or other things (such as the modem firmware) still needs to be researched as well.

Also, it's not clear how the AP tarball is transferred (for example, whether it's unpacked at all or just sent verbatim)

#### Target to flasher

The first Integer is always `0x66`.

### End session packet

#### Flasher to target

The first Integer is always `0x67`. This is followed by an action specifier:

* `0x00` "Don't reboot": Asks the target to just terminate the session.
* `0x01` "Reboot":  Asks the target to terminate the session and reboot to system.

Both Heimdall and Odin always send a packet indicating not to reboot before sending a reboot packet. Mot sure whether this is required or simply a programming quirk.

#### Target to flasher

The target responds to each packet sent by the flasher with an 8 byte long packet. The first Integer is always `0x67`. The second one appears to be always `0x00`, even if the reboot flag (so `0x01`) was sent.

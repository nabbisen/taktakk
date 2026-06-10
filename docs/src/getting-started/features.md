# What is taktakk?

taktakk is an offline-first educational platform for communities in conflict
zones, refugee camps, and areas with no reliable network or power supply.

## Core features

**Clock facade**
The app presents as a plain clock with alarm, stopwatch, and timer. No
product name, no app icon hint, and no learning content is visible until
the user performs the unlock gesture.

**Stealth unlock**
Unlocking works by setting an alarm to a specific time and holding the
Confirm button. The gesture is indistinguishable from ordinary clock use.
A separate duress code silently wipes the device.

**Shield / Spear curriculum**
Two learning axes deliver knowledge that matters:
- *Shield* — survival: water safety, first aid, navigation, device security.
- *Spear* — empowerment: maths, logic, communication, digital literacy.

**Offline-only operation**
No internet connection is ever required or requested. Content travels between
devices via Bluetooth, Wi-Fi Direct, SD card, or USB. The app never connects
to any server.

**Lesson viewer**
Each lesson step shows a pictogram or SVG illustration with optional audio
narration. Interactive drills check understanding. Progress is saved after
every step so a power outage wastes at most one step.

**Instant wipe**
When the duress gesture is entered, the app overwrites all cryptographic keys
in under a second. Without the keys, all lesson content becomes unreadable
even if the device is imaged. The app then resets to a plain unconfigured
clock.

**Signed content packages**
Every module arrives as a `.nmp` file signed with Ed25519. The app verifies
the signature before extracting any content. Tampered or unsigned packages
are quarantined and never executed.

**Zero telemetry**
No analytics. No crash reporters. No network pings. Learning progress never
leaves the device unless the user explicitly exports it.

## Accessibility

taktakk is designed to the ABDD (Accessible by Design) standard:

- Large touch targets (≥ 48 dp) are on by default.
- High-contrast dark theme suitable for bright sunlight.
- Audio narration for every lesson step.
- Full RTL layout for Arabic, Farsi, Urdu, and related scripts.
- Reduced motion by default to save battery.
- Text scale adjustable from 0.8× to 3.0×.

## Supported environments

| Constraint | taktakk's response |
|---|---|
| No internet | Fully offline; P2P and physical media only |
| Low battery / solar charging | Minimal background activity; no video |
| Old hardware (1 GB RAM, ARMv7) | Binary target ≤ 50 MB; SQLite, not a cloud DB |
| Multiple languages + RTL | BCP 47 locale packs; 3-tier fallback lookup |
| Device inspection risk | Clock facade; instant crypto erasure |

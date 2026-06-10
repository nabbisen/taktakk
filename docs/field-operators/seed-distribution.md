# Seed Distribution Guide

This guide is for field operators distributing taktakk to communities.

## What you need

- One Android device with taktakk installed (the "seed device").
- One or more SD cards or USB drives (minimum 256 MB each).
- Optional: a laptop running `taktakk-linux` for content preparation.

## Preparing seed media

1. On the seed device, unlock taktakk and go to Share → Export to media.
2. Insert an SD card.
3. Tap "Export all packages" and wait for the progress indicator to finish.
4. The SD card now contains all installed `.nmp` packages.
5. Safely eject the card.

## Distributing to a new device

1. Install the taktakk APK on the new device (sideload via USB or SD).
2. Insert the seed SD card.
3. Open the taktakk clock, unlock it with the default gesture.
4. Go to Import → Scan storage card.
5. Tap each package listed and confirm "Import".
6. Verify the package list matches what you expected.

## Safety rules

- **Never** write the unlock sequence on paper near a device.
- **Always** configure the duress code before distributing to users.
- **Never** include personally identifiable information in package names.
- If a device is seized, the user should use the duress code immediately.
- Operators should not know individual users' unlock sequences.

## Verifying a package

taktakk verifies Ed25519 signatures automatically. If a package shows
"Quarantined", it was tampered with or signed by an untrusted key.
Never import quarantined packages to other devices.

## Updating content

When new packages are available:

1. Receive the update package on any connected device.
2. Share it peer-to-peer or via SD card as above.
3. Users will see a "New version available" indicator in the module list.

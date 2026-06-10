# Field Pilot Checklist (RFC 033)

## Before starting a pilot

### Device preparation
- [ ] All pilot devices are running the latest release candidate APK.
- [ ] The duress code is configured on each device.
- [ ] The factory-reset path is tested on at least one device.
- [ ] No personal data is pre-loaded on devices.
- [ ] Storage health check passes on all devices (`> 100 MB free`).

### Content preparation
- [ ] Seed kit profile selected (Minimal / Standard / Full).
- [ ] All packages in the kit verify with valid signatures.
- [ ] At least one locale pack is installed (RTL + LTR if applicable).
- [ ] Operator has read the module content and confirmed accuracy.

### Operator training
- [ ] Operators know the unlock gesture (but not each other's duress code).
- [ ] Operators know the stop conditions (see below).
- [ ] Operators have printed feedback forms (non-identifying fields only).
- [ ] Operators have the offline emergency procedure card.

## Stop conditions

Stop the pilot immediately if:
- A panic wipe fails or appears incomplete.
- Any internal route is visible without unlocking.
- A package signature check is bypassed.
- A user reports the app increases inspection risk.
- Any telemetry is observed leaving the device network.

## Allowed feedback (no personal data)

Operators may collect:
- Aggregated completion counts per module (e.g. "12 of 15 participants completed lesson 3").
- Verification failure error codes.
- Accessibility issues observed (e.g. "text too small on Device Class A").
- Device model class (e.g. "Android 9, 1 GB RAM, 32 GB storage").

Operators must NOT collect:
- Names, phone numbers, or any identifying information.
- Raw exercise answers or learning sequences.
- Peer relationship data.
- Location data of any kind.

## After the pilot

- [ ] All feedback forms collected and converted to non-identifying issue reports.
- [ ] Devices wiped or returned to factory reset state.
- [ ] Any packages that failed verification reported to the maintainer with error code only.
- [ ] Pilot results documented: proceed / revise / withdraw.

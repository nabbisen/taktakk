# FAQ

## General

**Does taktakk need internet access?**
Never. All learning content is stored locally. Modules travel between devices
via Bluetooth, Wi-Fi Direct, SD card, or USB. No server is ever contacted.

**What languages does taktakk support?**
The app supports LTR and RTL layouts. Starter locale packs cover English (en),
Arabic (ar), and Swahili (sw). Additional locale packs can be added as
`.nmp` packages through the same import flow as content.

**Can I use taktakk without audio?**
Yes. Every step has a pictogram or SVG illustration. Audio is optional and
can be turned off in Settings. Captions are available when audio is played.

**How much storage does taktakk need?**
The core app binary is targeted at ≤ 50 MB. Each content module is typically
1–10 MB depending on SVG complexity and audio length. The app needs at least
100 MB free to operate normally.

---

## Content and learning

**Why can't I see any modules after installing?**
You need to import content packages first. Use **Share → Import from storage**
to load `.nmp` files from an SD card, or receive them from another device
via **Share**.

**I was in the middle of a lesson when the phone died. Did I lose my progress?**
No. taktakk saves your position after every completed step. When you reopen
the module, it resumes exactly where you left off.

**A module shows "Quarantined". What does this mean?**
The package failed its Ed25519 signature or SHA-256 content check. This means
the file may have been tampered with or is corrupt. Do not use it. Report the
error code to whoever provided the package.

**Can I go back to a previous step?**
Yes. Tap the back arrow inside a lesson to return to the previous step.

---

## Security and privacy

**Who can see my learning progress?**
Nobody except you. Progress is stored only on your device and never transmitted
to any server. It is encrypted with a local key that is wiped along with the
content when you enter the duress code.

**What happens when I enter the duress code?**
The app immediately overwrites all cryptographic keys. Without those keys,
all content — lessons, progress, and module data — is permanently unreadable.
The app then resets to a plain clock. There is no visible confirmation.

**Can someone restore the deleted data?**
No. taktakk uses cryptographic erasure: only the key is destroyed, but without
it the data is computationally unrecoverable. Slow file deletion is not relied
upon for security.

**Is my identity stored anywhere?**
No. taktakk does not collect names, phone numbers, email addresses, or any
identifying information. The local profile is an anonymous random identifier
used only to track your own progress.

**Does taktakk have analytics or telemetry?**
No. Zero telemetry. The only log kept is a 24-hour rolling event log of
anonymous operational events (session open/close, install ok/fail). It contains
no module names, no user identifiers, and no content details.

---

## Technical

**Which Android versions does taktakk support?**
Android 8.0 (API level 26) and above. It is optimised for devices with 1 GB
RAM and ARM processors common in 5–10-year-old phones.

**Why does the app ask for Bluetooth / network permissions?**
Only when you open the Share menu. Permissions are never requested at startup.
The explanation shown to you will not mention "taktakk" or any learning context.

**Can I verify that the app binary matches the source code?**
Yes. See the [Reproducible Builds](../contributing/reproducible-builds.md)
guide for SHA-256 checksum verification instructions.

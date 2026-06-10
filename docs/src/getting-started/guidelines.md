# Usage Guidelines

## Before distributing taktakk

### Configure the unlock and duress codes

The default gesture (**03:14 long-press**) is documented publicly and must
be changed before real deployment.

1. Unlock the app with the current gesture.
2. Go to **Settings → Security**.
3. Set a new alarm time as the unlock trigger.
4. Set a *different* alarm time as the duress trigger.
5. Confirm both. Neither is stored as plaintext.

> Keep the duress code simple and memorable under stress. It should be
> easy to enter quickly. Do not write it near the device.

### Run the health check

After setting up a seed device, unlock the app and run the health check
from **Settings → Device health**. Confirm:

- At least one Shield module is installed.
- At least one locale pack is installed.
- Storage has more than 100 MB free.
- All installed packages show "Verified".

### Test the wipe path

Before deploying, test the duress wipe on a non-critical device:

1. Enter the duress alarm time + long-press.
2. Confirm the app returns to an unconfigured clock with no content.
3. Reinstall your content from the seed kit.

## Field distribution

### Preparing seed media (SD card)

1. On the seed device, unlock and go to **Share → Export to storage**.
2. Insert a blank SD card and tap **Export all packages**.
3. Wait for the progress indicator to finish, then safely eject the card.

### Distributing to a new device

1. Install the taktakk APK by sideloading via USB or from the SD card.
2. Insert the seed SD card.
3. Unlock the app and go to **Share → Import from storage**.
4. Tap each package listed and confirm "Import".

### Safety rules

- **Never** write the unlock sequence near a device.
- **Never** include personal names in module titles or package IDs.
- Operators should not know individual users' unlock sequences.
- If a device may be seized, the user should enter the duress code immediately.
- Quarantined packages must never be moved to other devices.

## Emergency procedures

**If the device is about to be inspected:**

Enter the duress alarm time + long-press. The app silently clears all content
and returns to a plain clock. There is no progress bar or confirmation.

**If a package fails verification:**

The package is quarantined automatically. Do not try to reinstall it. Report
the failure code to the content maintainer (share the error code only, not
any user details).

**If the app stops responding:**

Force-close and reopen. The most recently completed lesson step is preserved.
No content is lost by a crash.

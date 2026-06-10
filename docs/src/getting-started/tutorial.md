# Tutorial

This tutorial walks through the primary user journey from first launch
to completing a lesson, and then through the safe sharing flow.

## Step 1 — First launch

Open the app. You will see a clock showing the current time, with buttons
for Alarm, Clock, Timer, and Stopwatch. Nothing else is visible.

This is intentional. The app looks like a standard clock to anyone watching.

## Step 2 — Unlock the learning platform

To access the learning content, you need to know the unlock gesture.

**Default gesture** (change this before distributing to users):

1. Tap **Alarm**.
2. Create a new alarm set to **03:14**.
3. Hold the **Save / Confirm** button for about **3 seconds**.

The app transitions to the learning home screen with the Shield and Spear
module lists.

> If you enter the alarm time configured as the *duress code* instead, the
> app silently wipes all content and returns to the clock. There is no
> confirmation dialog. See [Usage Guidelines](guidelines.md) for how to
> configure the duress code.

## Step 3 — Choose an axis

The home screen shows two panels: **Shield** and **Spear**.

- Tap **Shield** to see survival-related modules (water safety, first aid, …).
- Tap **Spear** to see empowerment modules (maths, communication, …).

Each module tile shows a title and a progress indicator.

## Step 4 — Open a module and start learning

1. Tap any module tile.
2. The first unfinished lesson opens automatically.
3. Read or listen to each step. Tap **Next** (or the forward arrow) to advance.
4. On exercise steps, choose an answer or acknowledge the content.
5. Your progress is saved automatically after each step.

**Returning after a power cut:** re-unlock the app and tap the same module.
The lesson reopens at the exact step where you left off.

## Step 5 — Share a module with another device

> Sharing requires opening the Share menu, which will request Bluetooth or
> local network permissions. These permissions are never requested at startup.

1. From the home screen, tap **Share**.
2. Grant the Bluetooth or network permission when asked.
3. The other device must also be in Share mode.
4. The app compares inventories and transfers only what is missing.
5. All transferred files are verified against their Ed25519 signatures before
   being installed.

## Step 6 — Import from an SD card

1. Insert the SD card containing `.nmp` package files.
2. From the home screen, tap **Share → Import from storage**.
3. Grant the storage permission when asked.
4. The app lists the packages it finds. Tap each one to import.
5. Packages are verified before installation; any that fail are quarantined.

## Step 7 — Return to the clock

Tap the back button or the clock icon in the navigation bar to return to
the facade clock. The app looks like a clock again.

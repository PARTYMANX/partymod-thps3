PARTYMOD 2.0

This is a patch for THPS3 1.01 to improve its input handling as well as smooth out a few other parts of the PC port.
The patch is designed to keep the game as original as possible, and leave its files unmodified.

Features and Fixes:
* Replaced input system entirely with new, modern system using the SDL3 library
* Movement stick now controls menus
* Restored controller vibration support
* Implemented controller support for park editor
* Added selectable controller glyphs for PC, PS2, Japanese PS2, and Xbox
* Improved cursor handling, no longer moving the cursor and only showing it when relevant
* Many graphics fixes bringing the game up to the visual standard of the console versions
* Rewrote the movie player so that it now scales up to the window size
* Improved window handling allowing for custom resolutions (now respecting scaled resolutions) and configurable windowing
* Fixed aspect ratio to be based on window dimensions (previously it was based on the PS2 framebuffer at 10:7/640x448)
* The game no longer opens the game's launcher when run directly
* Replaced configuration files with new INI-based system (see partymod.ini)
* Custom configurator program to handle new configuration files
* Fixed ledge warp bugs where the skater is teleported down farther than intended
* Fixes missing sounds after several retries
* Fixes voices and other streaming sounds always playing at the same volume
* Fixes music skipping tracks while the game is paused
* Connects to alternative online services (defaults to OpenSpy)
* Fixes network interface binding issues (hosting servers works now!  remember to forward ports 5150-5151 (as usual) as well as 6500)
* Optionally adds a single level practice mode for speedrunning (available via config option or `-ilmode` launch flag)
* Removes the trick cap for combo multipliers and graffiti tag limit (or enable compatibility mode to keep them)
* Allows multiple save files through the `-profile <profile>` launch argument
* Adds hotkey on F11 to disable the HUD for screenshots

INSTALLATION
NOTE: If upgrading from PARTYMOD 1.x, make sure to rerun the patcher!
1. Download PARTYMOD from the releases tab
2. Make sure THPS3 (English) is installed and the 1.01 patch is applied, remove the widescreen mod if it is installed (delete dinput8.dll)
3. Extract this zip folder into your THPS3 installation directory
4. Run partypatcher.exe to create the new, patched THPS3.exe game executable. This will be used to launch the game from now on. (NOTE: again, if upgrading from PARTYMOD 1.x, do this again!)
5. Optionally (highly recommended), configure the game with partyconfig.exe
6. Launch the game from THPS3.exe

NOTE: if the game is installed into the "Program Files" directory, you may need to run each program as administrator.
Also, if the game is installed into the "Program Files" directory, save files will be saved in the C:\Users\<name>\AppData\Local\VirtualStore directory.
For more information, see here: https://answers.microsoft.com/en-us/windows/forum/all/please-explain-virtualstore-for-non-experts/d8912f80-b275-48d7-9ff3-9e9878954227

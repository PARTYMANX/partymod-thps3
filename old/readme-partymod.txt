PARTYMOD 1.1.3

This is a patch for THPS3 1.01 to improve its input handling as well as smooth out a few other parts of the PC port.
The patch is designed to keep the game as original as possible, and leave its files unmodified.

Features and Fixes:
- Replaced input system entirely with new, modern system using the SDL2 library
- Movement stick now controls menus
- Improved cursor handling, no longer moving the cursor and only showing it when relevant
- Improved window handling allowing for custom resolutions and configurable windowing
- Fixed aspect ratio to be based on window dimensions (previously it was based on the PS2 framebuffer at 10:7/640x448)
- The game no longer opens the game's launcher when run directly
- Replaced configuration files with new INI-based system (see partymod.ini)
- Custom configurator program to handle new configuration files
- Fixed ledge warp bugs where the skater is teleported down farther than intended
- Fixed visually missing geometry in various areas (notably, the airport entrance and destructable wall in Skater Island)
- Fixes music randomization having the same sequence between sessions
- Fixes the skater's shadow not appearing transparent
- Fixes missing sounds after several retries
- Connects to alternative online services (defaults to OpenSpy)
- Fixes network interface binding issues (hosting servers works now!  remember to forward ports 5150-5151 (as usual) as well as 6500)
- Optionally adds a single level practice mode for speedrunning
- Optionally removes the trick cap for combo multipliers

INSTALLATION:
1. Make sure THPS3 (English) is installed and the 1.01 patch is applied, remove the widescreen mod if it is installed (delete dinput8.dll)
2. Extract this zip folder into your THPS3 installation directory
3. Run partypatcher.exe to create the new, patched THPS3.exe game executable (this will be used to launch the game from now on) (this only needs to be done once)
4. Optionally (highly recommended), configure the game with partyconfig.exe
5. Launch the game from THPS3.exe

NOTE: if the game is installed into the "Program Files" directory, you may need to run each program as administrator. 
Also, if the game is installed into the "Program Files" directory, save files will be saved in the C:\Users\<name>\AppData\Local\VirtualStore directory.  
For more information, see here: https://answers.microsoft.com/en-us/windows/forum/all/please-explain-virtualstore-for-non-experts/d8912f80-b275-48d7-9ff3-9e9878954227
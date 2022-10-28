#include <stdio.h>

#include <Windows.h>
#include <windowsx.h>
#include <CommCtrl.h>

#include <SDL2/SDL.h>

// configuration application for PARTYMOD
// sorry this code is so ugly, i made some early missteps and have discovered only a week in that microsoft's abysmal documentation 
// doesn't tell you the million pitfalls in the literal namesake of the operating system thus i had no motivation to fix it
// sorry the program is ugly too.  same deal.
// anyway, if you'd like to fix it, be my guest!

#pragma comment(linker,"\"/manifestdependency:type='win32' \
name='Microsoft.Windows.Common-Controls' version='6.0.0.0' \
processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'\"")

#define CONFIG_FILE_NAME "partymod.ini"

HWND windowHwnd;
HFONT hfont;
int cxMargin;
int cyMargin;

struct displayMode {
	int width;
	int height;
};

int numDisplayModes;
struct displayMode *displayModeList;

void initResolutionList() {
	DEVMODE deviceMode;

	numDisplayModes = 0;
	displayModeList = NULL;
	
	int i = 0;

	while (EnumDisplaySettings(NULL, i, &deviceMode)) {
		struct displayMode displayMode = { deviceMode.dmPelsWidth, deviceMode.dmPelsHeight };

		// insert into list
		if (displayModeList == NULL) {
			displayModeList = malloc(sizeof(struct displayMode));
			displayModeList[0] = displayMode;
			numDisplayModes = 1;
		} else {
			// search list for duplicate or larger resolution
			int j;
			int isDuplicate = 0;
			for (j = 0; j < numDisplayModes; j++) {
				if (displayModeList[j].width == displayMode.width && displayModeList[j].height == displayMode.height) {
					isDuplicate = 1;
					break;
				} else if (displayModeList[j].width >= displayMode.width && displayModeList[j].height >= displayMode.height) {
					break;
				}
			}

			if (!isDuplicate) {
				displayModeList = realloc(displayModeList, sizeof(struct displayMode) * (numDisplayModes + 1));
				
				// move existing elements
				for (int k = numDisplayModes; k > j; k--) {
					displayModeList[k] = displayModeList[k - 1];
				}

				displayModeList[j] = displayMode;
				numDisplayModes++;
			}
		}

		i++;
	}

	for (i = 0; i < numDisplayModes; i++) {
		printf("OUTPUT DISPLAY MODE %d: %d x %d\n", i, displayModeList[i].width, displayModeList[i].height);
	}
}

struct settings {
	int resX;
	int resY;
	int windowed;

	int shadows;
	int particles;
	int animatedTextures;
	int distanceFog;
	int lowDetail;

	int playIntro;
};

struct keybinds {
	SDL_Scancode ollie;
	SDL_Scancode grab;
	SDL_Scancode flip;
	SDL_Scancode grind;
	SDL_Scancode spinLeft;
	SDL_Scancode spinRight;
	SDL_Scancode nollie;
	SDL_Scancode switchRevert;
	SDL_Scancode pause;

	SDL_Scancode forward;
	SDL_Scancode backward;
	SDL_Scancode left;
	SDL_Scancode right;

	SDL_Scancode camUp;
	SDL_Scancode camDown;
	SDL_Scancode camLeft;
	SDL_Scancode camRight;
	SDL_Scancode viewToggle;
	SDL_Scancode swivelLock;
};

// a recreation of the SDL_GameControllerButton enum, but with the addition of right/left trigger
typedef enum {
	CONTROLLER_UNBOUND = -1,
	CONTROLLER_BUTTON_A = SDL_CONTROLLER_BUTTON_A,
	CONTROLLER_BUTTON_B = SDL_CONTROLLER_BUTTON_B,
	CONTROLLER_BUTTON_X = SDL_CONTROLLER_BUTTON_X,
	CONTROLLER_BUTTON_Y = SDL_CONTROLLER_BUTTON_Y,
	CONTROLLER_BUTTON_BACK = SDL_CONTROLLER_BUTTON_BACK,
	CONTROLLER_BUTTON_GUIDE = SDL_CONTROLLER_BUTTON_GUIDE,
	CONTROLLER_BUTTON_START = SDL_CONTROLLER_BUTTON_START,
	CONTROLLER_BUTTON_LEFTSTICK = SDL_CONTROLLER_BUTTON_LEFTSTICK,
	CONTROLLER_BUTTON_RIGHTSTICK = SDL_CONTROLLER_BUTTON_RIGHTSTICK,
	CONTROLLER_BUTTON_LEFTSHOULDER = SDL_CONTROLLER_BUTTON_LEFTSHOULDER,
	CONTROLLER_BUTTON_RIGHTSHOULDER = SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
	CONTROLLER_BUTTON_DPAD_UP = SDL_CONTROLLER_BUTTON_DPAD_UP,
	CONTROLLER_BUTTON_DPAD_DOWN = SDL_CONTROLLER_BUTTON_DPAD_DOWN,
	CONTROLLER_BUTTON_DPAD_LEFT = SDL_CONTROLLER_BUTTON_DPAD_LEFT,
	CONTROLLER_BUTTON_DPAD_RIGHT = SDL_CONTROLLER_BUTTON_DPAD_RIGHT,
	CONTROLLER_BUTTON_MISC1 = SDL_CONTROLLER_BUTTON_MISC1,
	CONTROLLER_BUTTON_PADDLE1 = SDL_CONTROLLER_BUTTON_PADDLE1,
	CONTROLLER_BUTTON_PADDLE2 = SDL_CONTROLLER_BUTTON_PADDLE2,
	CONTROLLER_BUTTON_PADDLE3 = SDL_CONTROLLER_BUTTON_PADDLE3,
	CONTROLLER_BUTTON_PADDLE4 = SDL_CONTROLLER_BUTTON_PADDLE4,
	CONTROLLER_BUTTON_TOUCHPAD = SDL_CONTROLLER_BUTTON_TOUCHPAD,
	CONTROLLER_BUTTON_RIGHTTRIGGER = 21,
	CONTROLLER_BUTTON_LEFTTRIGGER = 22,
} controllerButton;

typedef enum {
	CONTROLLER_STICK_UNBOUND = -1,
	CONTROLLER_STICK_LEFT = 0,
	CONTROLLER_STICK_RIGHT = 1
} controllerStick;

struct controllerbinds {
	controllerButton menu;
	controllerButton cameraToggle;
	controllerButton cameraSwivelLock;

	controllerButton grind;
	controllerButton grab;
	controllerButton ollie;
	controllerButton kick;

	controllerButton leftSpin;
	controllerButton rightSpin;
	controllerButton nollie;
	controllerButton switchRevert;

	controllerButton right;
	controllerButton left;
	controllerButton up;
	controllerButton down;

	controllerStick movement;
	controllerStick camera;
};

struct settings settings;
struct keybinds keybinds;
struct controllerbinds padbinds;

int getIniBool(char *section, char *key, int def, char *file) {
	int result = GetPrivateProfileInt(section, key, def, file);
	if (result) {
		return 1;
	} else {
		return 0;
	}
}

void writeIniBool(char *section, char *key, int val, char *file) {
	if (val) {
		WritePrivateProfileString(section, key, "1", file);
	} else {
		WritePrivateProfileString(section, key, "0", file);
	}
}

void writeIniInt(char *section, char *key, int val, char *file) {
	char buf[16];
	itoa(val, buf, 10);
	WritePrivateProfileString(section, key, buf, file);
}

void defaultSettings() {
	settings.resX = 640;
	settings.resY = 480;
	settings.windowed = 0;

	settings.shadows = 1;
	settings.particles = 1;
	settings.animatedTextures = 1;
	settings.distanceFog = 0;
	settings.lowDetail = 0;

	settings.playIntro = 1;

	keybinds.ollie = SDL_SCANCODE_KP_2;
	keybinds.grab = SDL_SCANCODE_KP_6;
	keybinds.flip = SDL_SCANCODE_KP_4;
	keybinds.grind = SDL_SCANCODE_KP_8;
	keybinds.spinLeft = SDL_SCANCODE_KP_1;
	keybinds.spinRight = SDL_SCANCODE_KP_3;
	keybinds.nollie = SDL_SCANCODE_KP_7;
	keybinds.switchRevert = SDL_SCANCODE_KP_9;
	keybinds.pause = 0;	// unbound

	keybinds.forward = SDL_SCANCODE_W;
	keybinds.backward = SDL_SCANCODE_S;
	keybinds.left = SDL_SCANCODE_A;
	keybinds.right = SDL_SCANCODE_D;

	keybinds.camUp = SDL_SCANCODE_I;
	keybinds.camDown = SDL_SCANCODE_K;
	keybinds.camLeft = SDL_SCANCODE_J;
	keybinds.camRight = SDL_SCANCODE_L;
	keybinds.viewToggle = SDL_SCANCODE_GRAVE;
	keybinds.swivelLock = 0;	// unbound

	padbinds.menu = CONTROLLER_BUTTON_START;
	padbinds.cameraToggle = CONTROLLER_BUTTON_BACK;
	padbinds.cameraSwivelLock = CONTROLLER_BUTTON_RIGHTSTICK;

	padbinds.grind = CONTROLLER_BUTTON_Y;
	padbinds.grab = CONTROLLER_BUTTON_B;
	padbinds.ollie = CONTROLLER_BUTTON_A;
	padbinds.kick = CONTROLLER_BUTTON_X;

	padbinds.leftSpin = CONTROLLER_BUTTON_LEFTSHOULDER;
	padbinds.rightSpin = CONTROLLER_BUTTON_RIGHTSHOULDER;
	padbinds.nollie = CONTROLLER_BUTTON_LEFTTRIGGER;
	padbinds.switchRevert = CONTROLLER_BUTTON_RIGHTTRIGGER;

	padbinds.right = CONTROLLER_BUTTON_DPAD_RIGHT;
	padbinds.left = CONTROLLER_BUTTON_DPAD_LEFT;
	padbinds.up = CONTROLLER_BUTTON_DPAD_UP;
	padbinds.down = CONTROLLER_BUTTON_DPAD_DOWN;

	padbinds.movement = CONTROLLER_STICK_LEFT;
	padbinds.camera = CONTROLLER_STICK_RIGHT;
}

char configFile[1024];

char *getConfigFile() {
	char executableDirectory[1024];
	int filePathBufLen = 1024;
	GetModuleFileName(NULL, &executableDirectory, filePathBufLen);

	// find last slash
	char *exe = strrchr(executableDirectory, '\\');
	if (exe) {
		*(exe + 1) = '\0';
	}

	sprintf(configFile, "%s%s", executableDirectory, CONFIG_FILE_NAME);

	return configFile;
}

void loadSettings() {
	char *configFile = getConfigFile();

	settings.resX = GetPrivateProfileInt("Graphics", "ResolutionX", 640, configFile);
	settings.resY = GetPrivateProfileInt("Graphics", "ResolutionY", 480, configFile);
	settings.windowed = getIniBool("Graphics", "Windowed", 0, configFile);

	settings.shadows = getIniBool("Graphics", "Shadows", 1, configFile);
	settings.particles = getIniBool("Graphics", "Particles", 1, configFile);
	settings.animatedTextures = getIniBool("Graphics", "AnimatedTextures", 1, configFile);
	settings.distanceFog = getIniBool("Graphics", "DistanceFog", 0, configFile);
	settings.lowDetail = getIniBool("Graphics", "LowDetailModels", 0, configFile);
   
	settings.playIntro = getIniBool("Miscellaneous", "PlayIntro", 1, configFile);

	keybinds.ollie = GetPrivateProfileInt("Keybinds", "Ollie", SDL_SCANCODE_KP_2, configFile);
	keybinds.grab = GetPrivateProfileInt("Keybinds", "Grab", SDL_SCANCODE_KP_6, configFile);
	keybinds.flip = GetPrivateProfileInt("Keybinds", "Flip", SDL_SCANCODE_KP_4, configFile);
	keybinds.grind = GetPrivateProfileInt("Keybinds", "Grind", SDL_SCANCODE_KP_8, configFile);
	keybinds.spinLeft = GetPrivateProfileInt("Keybinds", "SpinLeft", SDL_SCANCODE_KP_1, configFile);
	keybinds.spinRight = GetPrivateProfileInt("Keybinds", "SpinRight", SDL_SCANCODE_KP_3, configFile);
	keybinds.nollie = GetPrivateProfileInt("Keybinds", "Nollie", SDL_SCANCODE_KP_7, configFile);
	keybinds.switchRevert = GetPrivateProfileInt("Keybinds", "Switch", SDL_SCANCODE_KP_9, configFile);
	keybinds.pause = GetPrivateProfileInt("Keybinds", "Pause", 0, configFile);

	keybinds.forward = GetPrivateProfileInt("Keybinds", "Forward", SDL_SCANCODE_W, configFile);
	keybinds.backward = GetPrivateProfileInt("Keybinds", "Backward", SDL_SCANCODE_S, configFile);
	keybinds.left = GetPrivateProfileInt("Keybinds", "Left", SDL_SCANCODE_A, configFile);
	keybinds.right = GetPrivateProfileInt("Keybinds", "Right", SDL_SCANCODE_D, configFile);

	keybinds.camUp = GetPrivateProfileInt("Keybinds", "CameraUp", SDL_SCANCODE_I, configFile);
	keybinds.camDown = GetPrivateProfileInt("Keybinds", "CameraDown", SDL_SCANCODE_K, configFile);
	keybinds.camLeft = GetPrivateProfileInt("Keybinds", "CameraLeft", SDL_SCANCODE_J, configFile);
	keybinds.camRight = GetPrivateProfileInt("Keybinds", "CameraRight", SDL_SCANCODE_L, configFile);
	keybinds.viewToggle = GetPrivateProfileInt("Keybinds", "ViewToggle", SDL_SCANCODE_GRAVE, configFile);
	keybinds.swivelLock = GetPrivateProfileInt("Keybinds", "SwivelLock", 0, configFile);

	padbinds.menu = GetPrivateProfileInt("Gamepad", "Pause", CONTROLLER_BUTTON_START, configFile);
	padbinds.cameraToggle = GetPrivateProfileInt("Gamepad", "ViewToggle", CONTROLLER_BUTTON_BACK, configFile);
	padbinds.cameraSwivelLock = GetPrivateProfileInt("Gamepad", "SwivelLock", CONTROLLER_BUTTON_RIGHTSTICK, configFile);

	padbinds.grind = GetPrivateProfileInt("Gamepad", "Grind", CONTROLLER_BUTTON_Y, configFile);
	padbinds.grab = GetPrivateProfileInt("Gamepad", "Grab", CONTROLLER_BUTTON_B, configFile);
	padbinds.ollie = GetPrivateProfileInt("Gamepad", "Ollie", CONTROLLER_BUTTON_A, configFile);
	padbinds.kick = GetPrivateProfileInt("Gamepad", "Flip", CONTROLLER_BUTTON_X, configFile);

	padbinds.leftSpin = GetPrivateProfileInt("Gamepad", "SpinLeft", CONTROLLER_BUTTON_LEFTSHOULDER, configFile);
	padbinds.rightSpin = GetPrivateProfileInt("Gamepad", "SpinRight", CONTROLLER_BUTTON_RIGHTSHOULDER, configFile);
	padbinds.nollie = GetPrivateProfileInt("Gamepad", "Nollie", CONTROLLER_BUTTON_LEFTTRIGGER, configFile);
	padbinds.switchRevert = GetPrivateProfileInt("Gamepad", "Switch", CONTROLLER_BUTTON_RIGHTTRIGGER, configFile);

	padbinds.right = GetPrivateProfileInt("Gamepad", "Right", CONTROLLER_BUTTON_DPAD_RIGHT, configFile);
	padbinds.left = GetPrivateProfileInt("Gamepad", "Left", CONTROLLER_BUTTON_DPAD_LEFT, configFile);
	padbinds.up = GetPrivateProfileInt("Gamepad", "Forward", CONTROLLER_BUTTON_DPAD_UP, configFile);
	padbinds.down = GetPrivateProfileInt("Gamepad", "Backward", CONTROLLER_BUTTON_DPAD_DOWN, configFile);

	padbinds.movement = GetPrivateProfileInt("Gamepad", "MovementStick", CONTROLLER_STICK_LEFT, configFile);
	padbinds.camera = GetPrivateProfileInt("Gamepad", "CameraStick", CONTROLLER_STICK_RIGHT, configFile);
}

void saveSettings() {
	char *configFile = getConfigFile();

	if (settings.resX < 640) {
		settings.resX = 640;
	}
	if (settings.resY < 480) {
		settings.resY = 480;
	}

	writeIniInt("Graphics", "ResolutionX", settings.resX, configFile);
	writeIniInt("Graphics", "ResolutionY", settings.resY, configFile);
	writeIniBool("Graphics", "Windowed", settings.windowed, configFile);

	writeIniBool("Graphics", "Shadows", settings.shadows, configFile);
	writeIniBool("Graphics", "Particles", settings.particles, configFile);
	writeIniBool("Graphics", "AnimatedTextures", settings.animatedTextures, configFile);
	writeIniBool("Graphics", "DistanceFog", settings.distanceFog, configFile);
	writeIniBool("Graphics", "LowDetailModels", settings.lowDetail, configFile);

	writeIniBool("Miscellaneous", "PlayIntro", settings.playIntro, configFile);

	writeIniInt("Keybinds", "Ollie", keybinds.ollie, configFile);
	writeIniInt("Keybinds", "Grab", keybinds.grab, configFile);
	writeIniInt("Keybinds", "Flip", keybinds.flip, configFile);
	writeIniInt("Keybinds", "Grind", keybinds.grind, configFile);
	writeIniInt("Keybinds", "SpinLeft", keybinds.spinLeft, configFile);
	writeIniInt("Keybinds", "SpinRight", keybinds.spinRight, configFile);
	writeIniInt("Keybinds", "Nollie", keybinds.nollie, configFile);
	writeIniInt("Keybinds", "Switch", keybinds.switchRevert, configFile);
	writeIniInt("Keybinds", "Pause", keybinds.pause, configFile);

	writeIniInt("Keybinds", "Forward", keybinds.forward, configFile);
	writeIniInt("Keybinds", "Backward", keybinds.backward, configFile);
	writeIniInt("Keybinds", "Left", keybinds.left, configFile);
	writeIniInt("Keybinds", "Right", keybinds.right, configFile);

	writeIniInt("Keybinds", "CameraUp", keybinds.camUp, configFile);
	writeIniInt("Keybinds", "CameraDown", keybinds.camDown, configFile);
	writeIniInt("Keybinds", "CameraLeft", keybinds.camLeft, configFile);
	writeIniInt("Keybinds", "CameraRight", keybinds.camRight, configFile);
	writeIniInt("Keybinds", "ViewToggle", keybinds.viewToggle, configFile);
	writeIniInt("Keybinds", "SwivelLock", keybinds.swivelLock, configFile);

	writeIniInt("Gamepad", "Pause", padbinds.menu, configFile);
	writeIniInt("Gamepad", "ViewToggle", padbinds.cameraToggle, configFile);
	writeIniInt("Gamepad", "SwivelLock", padbinds.cameraSwivelLock, configFile);

	writeIniInt("Gamepad", "Grind", padbinds.grind, configFile);
	writeIniInt("Gamepad", "Grab", padbinds.grab, configFile);
	writeIniInt("Gamepad", "Ollie", padbinds.ollie, configFile);
	writeIniInt("Gamepad", "Flip", padbinds.kick, configFile);

	writeIniInt("Gamepad", "SpinLeft", padbinds.leftSpin, configFile);
	writeIniInt("Gamepad", "SpinRight", padbinds.rightSpin, configFile);
	writeIniInt("Gamepad", "Nollie", padbinds.nollie, configFile);
	writeIniInt("Gamepad", "Switch", padbinds.switchRevert, configFile);

	writeIniInt("Gamepad", "Right", padbinds.right, configFile);
	writeIniInt("Gamepad", "Left", padbinds.left, configFile);
	writeIniInt("Gamepad", "Forward", padbinds.up, configFile);
	writeIniInt("Gamepad", "Backward", padbinds.down, configFile);

	writeIniInt("Gamepad", "MovementStick", padbinds.movement, configFile);
	writeIniInt("Gamepad", "CameraStick", padbinds.camera, configFile);
}

void cursedSDLSetup() {
	// in order to key names, SDL has to have created a window
	// this function achieves that to initialize key binds

	SDL_Init(SDL_INIT_EVENTS);

	SDL_Window *win = SDL_CreateWindow("uhh... you're not supposed to see this", 0, 0, 0, 0, SDL_WINDOW_HIDDEN);

	SDL_DestroyWindow(win);

	SDL_Quit();
}

void setBindText(HWND box, SDL_Scancode key) {
	// NOTE: requires SDL to be initialized
	if (key == 0) {
		Edit_SetText(box, "Unbound");
	} else {
		Edit_SetText(box, SDL_GetKeyName(SDL_GetKeyFromScancode(key)));
	}
}

void doKeyBind(char *name, SDL_Scancode *target, HWND box) {
	Edit_SetText(box, "Press key...");

	SDL_Init(SDL_INIT_EVENTS);

	char namebuf[64];
	sprintf(namebuf, "Press Key To Bind To %s", name);

	RECT windowRect;
	if (!GetWindowRect(windowHwnd, &windowRect)) {
		printf("Failed to get window rect!!\n");
		return;
	}

	SDL_Window *inputWindow = SDL_CreateWindow(namebuf, windowRect.left, windowRect.top, windowRect.right - windowRect.left, windowRect.bottom - windowRect.top, SDL_WINDOW_INPUT_FOCUS | SDL_WINDOW_BORDERLESS);

	if (!inputWindow) {
		printf("%s\n", SDL_GetError());
		SDL_Quit();
		return;
	}

	int doneInput = 0;
	SDL_Event e;
	while (!doneInput) {
		while(SDL_PollEvent(&e) && !doneInput) {
			if (e.type == SDL_KEYDOWN) {
				SDL_KeyCode keyCode = SDL_GetKeyFromScancode(e.key.keysym.scancode);
				printf("Pressed key %s\n", SDL_GetKeyName(keyCode));
				*target = e.key.keysym.scancode;
					
				doneInput = 1;
			} else if (e.type == SDL_QUIT) {
				doneInput = 1;
			} else if (e.type == SDL_WINDOWEVENT) {
				if (e.window.event == SDL_WINDOWEVENT_HIDDEN || e.window.event == SDL_WINDOWEVENT_FOCUS_LOST || e.window.event == SDL_WINDOWEVENT_MINIMIZED) {
					doneInput = 1;
				}
			}
		}
	}
	SDL_DestroyWindow(inputWindow);

	setBindText(box, *target);

	SDL_Quit();
}

struct settingsPage {
	HWND page;

	HWND resolutionGroup;
	HWND graphicsGroup;
	HWND miscGroup;

	HWND resolutionComboBox;
	HWND customResolutionCheckbox;
	HWND customResolutionXLabel;
	HWND customResolutionXBox;
	HWND customResolutionYLabel;
	HWND customResolutionYBox;
	HWND windowedCheckbox;

	HWND shadowCheckbox;
	HWND particleCheckbox;
	HWND animatedTexturesCheckbox;
	HWND distanceFogCheckbox;
	HWND lowDetailCheckbox;

	HWND introCheckbox;
};

struct keyboardPage {
	HWND page;

	HWND actionGroup;
	HWND movementGroup;
	HWND cameraGroup;

	HWND ollieLabel;
	HWND ollieBox;
	HWND grabLabel;
	HWND grabBox;
	HWND flipLabel;
	HWND flipBox;
	HWND grindLabel;
	HWND grindBox;
	HWND spinLLabel;
	HWND spinLBox;
	HWND spinRLabel;
	HWND spinRBox;
	HWND nollieLabel;
	HWND nollieBox;
	HWND switchLabel;
	HWND switchBox;
	HWND pauseLabel;
	HWND pauseBox;

	HWND forwardLabel;
	HWND forwardBox;
	HWND backwardLabel;
	HWND backwardBox;
	HWND leftLabel;
	HWND leftBox;
	HWND rightLabel;
	HWND rightBox;
	
	HWND camUpLabel;
	HWND camUpBox;
	HWND camDownLabel;
	HWND camDownBox;
	HWND camLeftLabel;
	HWND camLeftBox;
	HWND camRightLabel;
	HWND camRightBox;
	HWND viewToggleLabel;
	HWND viewToggleBox;
	HWND swivelLockLabel;
	HWND swivelLockBox;
};

struct gamepadPage {
	HWND page;

	HWND actionGroup;
	HWND movementGroup;
	HWND cameraGroup;

	HWND ollieLabel;
	HWND ollieBox;
	HWND grabLabel;
	HWND grabBox;
	HWND flipLabel;
	HWND flipBox;
	HWND grindLabel;
	HWND grindBox;
	HWND spinLLabel;
	HWND spinLBox;
	HWND spinRLabel;
	HWND spinRBox;
	HWND nollieLabel;
	HWND nollieBox;
	HWND switchLabel;
	HWND switchBox;
	HWND pauseLabel;
	HWND pauseBox;

	HWND forwardLabel;
	HWND forwardBox;
	HWND backwardLabel;
	HWND backwardBox;
	HWND leftLabel;
	HWND leftBox;
	HWND rightLabel;
	HWND rightBox;
	HWND moveStickLabel;
	HWND moveStickBox;
	
	HWND camStickLabel;
	HWND camStickBox;
	HWND viewToggleLabel;
	HWND viewToggleBox;
	HWND swivelLockLabel;
	HWND swivelLockBox;
};

struct settingsPage settingsPage;
struct keyboardPage keyboardPage;
struct gamepadPage gamepadPage;

#define CHECK_CUSTOMRESOLUTION 0x8801
#define CHECK_WINDOWED 0x8802
#define CHECK_SHADOW 0x8803
#define CHECK_PARTICLE 0x8804
#define CHECK_ANIMATEDTEXTURES 0x8805
#define CHECK_DISTANCEFOG 0x8806
#define CHECK_LOWDETAIL 0x8807
#define CHECK_INTRO 0x8808
#define COMBOBOX_RES 0x8809
#define FIELD_RESX 0x880a
#define FIELD_RESY 0x880b

int toggleCheck(HWND control) {
	int checkstate = SendMessage(control, BM_GETCHECK, 0, 0);
	if (checkstate == BST_UNCHECKED) {
		SendMessage(control, BM_SETCHECK, BST_CHECKED, 0);
		return 1;
	} else {
		SendMessage(control, BM_SETCHECK, BST_UNCHECKED, 0);
		return 0;
	}
}

LRESULT CALLBACK generalPageProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_COMMAND: {
			int controlId = LOWORD(wParam);

			if (controlId == CHECK_CUSTOMRESOLUTION) {
				if (toggleCheck(settingsPage.customResolutionCheckbox)) {
					// enable custom resolution boxes, disable combo box
					ComboBox_Enable(settingsPage.resolutionComboBox, FALSE);
					Static_Enable(settingsPage.customResolutionXLabel, TRUE);
					Edit_Enable(settingsPage.customResolutionXBox, TRUE);
					Static_Enable(settingsPage.customResolutionYLabel, TRUE);
					Edit_Enable(settingsPage.customResolutionYBox, TRUE);

					char entryBuffer[16];
					Edit_GetText(settingsPage.customResolutionXBox, entryBuffer, 16);
					settings.resX = atoi(entryBuffer);

					Edit_GetText(settingsPage.customResolutionYBox, entryBuffer, 16);
					settings.resY = atoi(entryBuffer);
				} else {
					// disable custom resolution boxes, enable combo box
					ComboBox_Enable(settingsPage.resolutionComboBox, TRUE);
					Static_Enable(settingsPage.customResolutionXLabel, FALSE);
					Edit_Enable(settingsPage.customResolutionXBox, FALSE);
					Static_Enable(settingsPage.customResolutionYLabel, FALSE);
					Edit_Enable(settingsPage.customResolutionYBox, FALSE);

					int idx = ComboBox_GetCurSel(settingsPage.resolutionComboBox);

					settings.resX = displayModeList[idx].width;
					settings.resY = displayModeList[idx].height;
				}
			} else if (controlId == COMBOBOX_RES) {
				if (HIWORD(wParam) == 1) {
					int idx = ComboBox_GetCurSel(lParam);

					settings.resX = displayModeList[idx].width;
					settings.resY = displayModeList[idx].height;
				}
			} else if (controlId == FIELD_RESX) {
				if (HIWORD(wParam) == EN_KILLFOCUS) {
					char entryBuffer[16];
					Edit_GetText(settingsPage.customResolutionXBox, entryBuffer, 16);
					settings.resX = atoi(entryBuffer);

					if (settings.resX < 320) {
						settings.resX = 320;
					}

					itoa(settings.resX, entryBuffer, 10);
					Edit_SetText(settingsPage.customResolutionXBox, entryBuffer);
				}
			} else if (controlId == FIELD_RESY) {
				if (HIWORD(wParam) == EN_KILLFOCUS) {
					char entryBuffer[16];
					Edit_GetText(settingsPage.customResolutionYBox, entryBuffer, 16);
					settings.resY = atoi(entryBuffer);

					if (settings.resY < 240) {
						settings.resY = 240;
					}

					itoa(settings.resY, entryBuffer, 10);
					Edit_SetText(settingsPage.customResolutionYBox, entryBuffer);
				}
			} else if (controlId == CHECK_WINDOWED) {
				settings.windowed = toggleCheck(settingsPage.windowedCheckbox);
			} else if (controlId == CHECK_SHADOW) {
				settings.shadows = toggleCheck(settingsPage.shadowCheckbox);
			} else if (controlId == CHECK_PARTICLE) {
				settings.particles = toggleCheck(settingsPage.particleCheckbox);
			} else if (controlId == CHECK_ANIMATEDTEXTURES) {
				settings.animatedTextures = toggleCheck(settingsPage.animatedTexturesCheckbox);
			} else if (controlId == CHECK_DISTANCEFOG) {
				settings.distanceFog = toggleCheck(settingsPage.distanceFogCheckbox);
			} else if (controlId == CHECK_LOWDETAIL) {
				settings.lowDetail = toggleCheck(settingsPage.lowDetailCheckbox);
			} else if (controlId == CHECK_INTRO) {
				settings.playIntro = toggleCheck(settingsPage.introCheckbox);
			}

			return 0;
		}
	}


	return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

HWND createSettingsPage(HWND parent, HINSTANCE hinst, RECT rect) {
	// TODO: create struct here
	//HWND result = CreateWindow(WC_STATIC, L"", WS_CHILD | WS_VISIBLE, rect.left, rect.top, rect.right, rect.bottom, parent, NULL, hinst, NULL);

	const char *className  = "General Page Class";

	WNDCLASS wc = { 0,
		generalPageProc,
		0,
		0,
		hinst,
		NULL,
		NULL,
		NULL,
		NULL,
		className
	};

	RegisterClass(&wc);

	HWND result = CreateWindow(className, L"", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, parent, NULL, hinst, NULL);
	// TODO: add handler that destroys struct

	RECT pageRect;
	GetClientRect(result, &pageRect);

	int pageWidth = pageRect.right - pageRect.left;
	int pageHeight = pageRect.bottom - pageRect.top;

	// RESOLUTION BOX
	int resolutionGroupX = 8;
	int resolutionGroupY = 8;
	settingsPage.resolutionGroup = CreateWindow(WC_BUTTON, "Resolution", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, resolutionGroupX, resolutionGroupY, pageWidth - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(settingsPage.resolutionGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	settingsPage.resolutionComboBox = CreateWindow(WC_COMBOBOX, "", WS_TABSTOP | WS_VISIBLE | WS_CHILD | CBS_DROPDOWNLIST, resolutionGroupX + 8, resolutionGroupY + 16, 200, 128, result, COMBOBOX_RES, hinst, NULL);
	SendMessage(settingsPage.resolutionComboBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.customResolutionCheckbox = CreateWindow(WC_BUTTON, "Use Custom Resolution", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, resolutionGroupX + 8, resolutionGroupY + 16 + 24, 200, 24, result, CHECK_CUSTOMRESOLUTION, hinst, NULL);
	SendMessage(settingsPage.customResolutionCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.customResolutionXLabel = CreateWindow(WC_STATIC, "Width:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE | SS_CENTER, resolutionGroupX + 16, resolutionGroupY + 16 + (24 * 2), 48, 24, result, NULL, hinst, NULL);
	SendMessage(settingsPage.customResolutionXLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.customResolutionXBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, resolutionGroupX + 16 + 48, resolutionGroupY + 16 + (24 * 2), 48, 24, result, FIELD_RESX, hinst, NULL);
	SendMessage(settingsPage.customResolutionXBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.customResolutionYLabel = CreateWindow(WC_STATIC, "Height:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE | SS_CENTER, resolutionGroupX + 16 + 96, resolutionGroupY + 16 + (24 * 2), 48, 24, result, NULL, hinst, NULL);
	SendMessage(settingsPage.customResolutionYLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.customResolutionYBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, resolutionGroupX + 16 + 144, resolutionGroupY + 16 + (24 * 2), 48, 24, result, FIELD_RESY, hinst, NULL);
	SendMessage(settingsPage.customResolutionYBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.windowedCheckbox = CreateWindow(WC_BUTTON, "Windowed", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, resolutionGroupX + 8, resolutionGroupY + 16 + (24 * 3), 200, 24, result, CHECK_WINDOWED, hinst, NULL);
	SendMessage(settingsPage.windowedCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);

	//ComboBox_AddItemData
	for (int i = 0; i < numDisplayModes; i++) {
		char resolutionString[64];
		sprintf(resolutionString, "%dx%d", displayModeList[i].width, displayModeList[i].height);
		ComboBox_AddString(settingsPage.resolutionComboBox, resolutionString);
	}
	ComboBox_SetCurSel(settingsPage.resolutionComboBox, 0);

	SetWindowPos(settingsPage.resolutionGroup, settingsPage.customResolutionYBox, resolutionGroupX, resolutionGroupY, pageWidth - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	// GRAPHICS BOX
	int graphicsGroupX = 8;
	int graphicsGroupY = (pageHeight / 2) + 8;
	HWND graphicsGroup = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Graphics", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, graphicsGroupX, graphicsGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(graphicsGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	settingsPage.shadowCheckbox = CreateWindow(WC_BUTTON, "Shadows", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, graphicsGroupX + 8, graphicsGroupY + 16, 150, 24, result, CHECK_SHADOW, hinst, NULL);
	SendMessage(settingsPage.shadowCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.particleCheckbox = CreateWindow(WC_BUTTON, "Particles", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, graphicsGroupX + 8, graphicsGroupY + 16 + 24, 150, 24, result, CHECK_PARTICLE, hinst, NULL);
	SendMessage(settingsPage.particleCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.animatedTexturesCheckbox = CreateWindow(WC_BUTTON, "Animating Textures", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, graphicsGroupX + 8, graphicsGroupY + 16 + (24 * 2), 150, 24, result, CHECK_ANIMATEDTEXTURES, hinst, NULL);
	SendMessage(settingsPage.animatedTexturesCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.distanceFogCheckbox = CreateWindow(WC_BUTTON, "Distance Fog", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, graphicsGroupX + 8, graphicsGroupY + 16 + (24 * 3), 150, 24, result, CHECK_DISTANCEFOG, hinst, NULL);
	SendMessage(settingsPage.distanceFogCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);
	settingsPage.lowDetailCheckbox = CreateWindow(WC_BUTTON, "Low Detail Models", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, graphicsGroupX + 8, graphicsGroupY + 16 + (24 * 4), 150, 24, result, CHECK_LOWDETAIL, hinst, NULL);
	SendMessage(settingsPage.lowDetailCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);

	// MISC BOX
	int miscGroupX = (pageWidth / 2) + 8;
	int miscGroupY = (pageHeight / 2) + 8;
	settingsPage.miscGroup = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Miscellaneous", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, miscGroupX, miscGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(settingsPage.miscGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	settingsPage.introCheckbox = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Always Play Intro", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, miscGroupX + 8, miscGroupY + 16, 150, 24, result, CHECK_INTRO, hinst, NULL);
	SendMessage(settingsPage.introCheckbox, WM_SETFONT, (WPARAM)hfont, TRUE);

	settingsPage.page = result;

	return result;
}

void updateSettingsPage() {
	int foundResolution = 0;
	for (int i = 0; i < numDisplayModes; i++) {
		if (displayModeList[i].width == settings.resX && displayModeList[i].height == settings.resY) {
			foundResolution = 1;
			ComboBox_SetCurSel(settingsPage.resolutionComboBox, i);

			Button_SetCheck(settingsPage.customResolutionCheckbox, 0);
			ComboBox_Enable(settingsPage.resolutionComboBox, TRUE);
			Static_Enable(settingsPage.customResolutionXLabel, FALSE);
			Edit_Enable(settingsPage.customResolutionXBox, FALSE);
			Static_Enable(settingsPage.customResolutionYLabel, FALSE);
			Edit_Enable(settingsPage.customResolutionYBox, FALSE);

			break;
		}
	}

	if (!foundResolution) {
		char buf[16];
		itoa(settings.resX, buf, 10);
		Edit_SetText(settingsPage.customResolutionXBox, buf);
		itoa(settings.resY, buf, 10);
		Edit_SetText(settingsPage.customResolutionYBox, buf);

		Button_SetCheck(settingsPage.customResolutionCheckbox, 1);
		ComboBox_Enable(settingsPage.resolutionComboBox, FALSE);
		Static_Enable(settingsPage.customResolutionXLabel, TRUE);
		Edit_Enable(settingsPage.customResolutionXBox, TRUE);
		Static_Enable(settingsPage.customResolutionYLabel, TRUE);
		Edit_Enable(settingsPage.customResolutionYBox, TRUE);
	}

	Button_SetCheck(settingsPage.windowedCheckbox, (settings.windowed) ? BST_CHECKED : BST_UNCHECKED);

	Button_SetCheck(settingsPage.shadowCheckbox, (settings.shadows) ? BST_CHECKED : BST_UNCHECKED);
	Button_SetCheck(settingsPage.particleCheckbox, (settings.particles) ? BST_CHECKED : BST_UNCHECKED);
	Button_SetCheck(settingsPage.animatedTexturesCheckbox, (settings.animatedTextures) ? BST_CHECKED : BST_UNCHECKED);
	Button_SetCheck(settingsPage.distanceFogCheckbox, (settings.distanceFog) ? BST_CHECKED : BST_UNCHECKED);
	Button_SetCheck(settingsPage.lowDetailCheckbox, (settings.lowDetail) ? BST_CHECKED : BST_UNCHECKED);

	Button_SetCheck(settingsPage.introCheckbox, (settings.playIntro) ? BST_CHECKED : BST_UNCHECKED);
}

#define FIELD_OLLIE 0x8801
#define FIELD_GRAB 0x8802
#define FIELD_FLIP 0x8803
#define FIELD_GRIND 0x8804
#define FIELD_SPINL 0x8805
#define FIELD_SPINR 0x8806
#define FIELD_NOLLIE 0x8807
#define FIELD_SWITCH 0x8808
#define FIELD_PAUSE 0x8809
#define FIELD_FORWARD 0x880a
#define FIELD_BACKWARD 0x880b
#define FIELD_LEFT 0x880c
#define FIELD_RIGHT 0x880d
#define FIELD_CAMUP 0x880e
#define FIELD_CAMDOWN 0x880f
#define FIELD_CAMLEFT 0x8810
#define FIELD_CAMRIGHT 0x8811
#define FIELD_VIEWTOGGLE 0x8812
#define FIELD_SWIVELLOCK 0x8813

void setAllBindText() {
	setBindText(keyboardPage.ollieBox, keybinds.ollie);
	setBindText(keyboardPage.grabBox, keybinds.grab);
	setBindText(keyboardPage.flipBox, keybinds.flip);
	setBindText(keyboardPage.grindBox, keybinds.grind);
	setBindText(keyboardPage.spinLBox, keybinds.spinLeft);
	setBindText(keyboardPage.spinRBox, keybinds.spinRight);
	setBindText(keyboardPage.nollieBox, keybinds.nollie);
	setBindText(keyboardPage.switchBox, keybinds.switchRevert);
	setBindText(keyboardPage.pauseBox, keybinds.pause);

	setBindText(keyboardPage.forwardBox, keybinds.forward);
	setBindText(keyboardPage.backwardBox, keybinds.backward);
	setBindText(keyboardPage.leftBox, keybinds.left);
	setBindText(keyboardPage.rightBox, keybinds.right);

	setBindText(keyboardPage.camUpBox, keybinds.camUp);
	setBindText(keyboardPage.camDownBox, keybinds.camDown);
	setBindText(keyboardPage.camLeftBox, keybinds.camLeft);
	setBindText(keyboardPage.camRightBox, keybinds.camRight);
	setBindText(keyboardPage.viewToggleBox, keybinds.viewToggle);
	setBindText(keyboardPage.swivelLockBox, keybinds.swivelLock);
}

LRESULT CALLBACK keyboardPageProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg)
	{
		case WM_COMMAND: {
			int controlId = LOWORD(wParam);
			int controlCode = HIWORD(wParam);

			if (controlCode == EN_SETFOCUS) {
				switch (controlId) {
				case FIELD_OLLIE:
					doKeyBind("Ollie", &keybinds.ollie, keyboardPage.ollieBox);
					break;
				case FIELD_GRAB:
					doKeyBind("Grab", &keybinds.grab, keyboardPage.grabBox);
					break;
				case FIELD_FLIP:
					doKeyBind("Flip", &keybinds.flip, keyboardPage.flipBox);
					break;
				case FIELD_GRIND:
					doKeyBind("Grind", &keybinds.grind, keyboardPage.grindBox);
					break;
				case FIELD_SPINL:
					doKeyBind("Spin Left", &keybinds.spinLeft, keyboardPage.spinLBox);
					break;
				case FIELD_SPINR:
					doKeyBind("Spin Right", &keybinds.spinRight, keyboardPage.spinRBox);
					break;
				case FIELD_NOLLIE:
					doKeyBind("Nollie", &keybinds.nollie, keyboardPage.nollieBox);
					break;
				case FIELD_SWITCH:
					doKeyBind("Switch", &keybinds.switchRevert, keyboardPage.switchBox);
					break;
				case FIELD_PAUSE:
					doKeyBind("Pause", &keybinds.pause, keyboardPage.pauseBox);
					break;
				case FIELD_FORWARD:
					doKeyBind("Forward", &keybinds.forward, keyboardPage.forwardBox);
					break;
				case FIELD_BACKWARD:
					doKeyBind("Backward", &keybinds.backward, keyboardPage.backwardBox);
					break;
				case FIELD_LEFT:
					doKeyBind("Left", &keybinds.left, keyboardPage.leftBox);
					break;
				case FIELD_RIGHT:
					doKeyBind("Right", &keybinds.right, keyboardPage.rightBox);
					break;
				case FIELD_CAMUP:
					doKeyBind("Camera Up", &keybinds.camUp, keyboardPage.camUpBox);
					break;
				case FIELD_CAMDOWN:
					doKeyBind("Camera Down", &keybinds.camDown, keyboardPage.camDownBox);
					break;
				case FIELD_CAMLEFT:
					doKeyBind("Camera Left", &keybinds.camLeft, keyboardPage.camLeftBox);
					break;
				case FIELD_CAMRIGHT:
					doKeyBind("Camera Right", &keybinds.camRight, keyboardPage.camRightBox);
					break;
				case FIELD_VIEWTOGGLE:
					doKeyBind("View Toggle", &keybinds.viewToggle, keyboardPage.viewToggleBox);
					break;
				case FIELD_SWIVELLOCK:
					doKeyBind("Swivel Lock", &keybinds.swivelLock, keyboardPage.swivelLockBox);
					break;
				default:
					break;
				}
			}

			return 0;
		}
	}


	return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

HWND createKeyboardPage(HWND parent, HINSTANCE hinst, RECT rect) {
	// TODO: create struct here
	//HWND result = CreateWindow(WC_STATIC, L"", WS_CHILD | WS_VISIBLE, rect.left, rect.top, rect.right, rect.bottom, parent, NULL, hinst, NULL);

	const char *className = "Keyboard Page Class";

	WNDCLASS wc = { 0,
		keyboardPageProc,
		0,
		0,
		hinst,
		NULL,
		NULL,
		NULL,
		NULL,
		className
	};

	RegisterClass(&wc);

	HWND result = CreateWindow(className, L"", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, parent, NULL, hinst, NULL);
	// TODO: add handler that destroys struct

	RECT pageRect;
	GetClientRect(result, &pageRect);

	int pageWidth = pageRect.right - pageRect.left;
	int pageHeight = pageRect.bottom - pageRect.top;

	// SKATER BOX
	int resolutionGroupX = 8;
	int resolutionGroupY = 8;
	int boxHeight = 20;
	int labelHPos = resolutionGroupX + 8;
	int boxHPos = resolutionGroupX + ((pageWidth / 2) - 16 - 8 - 64);
	int boxVSpacing = 32;

	keyboardPage.actionGroup = CreateWindow(WC_BUTTON, "Actions", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, resolutionGroupX, resolutionGroupY, (pageWidth / 2) - 16, pageHeight - 16, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.actionGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	keyboardPage.ollieLabel = CreateWindow(WC_STATIC, "Ollie:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16, 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.ollieLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.ollieBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16, 64, boxHeight, result, (HMENU)FIELD_OLLIE, hinst, NULL);
	SendMessage(keyboardPage.ollieBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.grabLabel = CreateWindow(WC_STATIC, "Grab:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.grabLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.grabBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, (HMENU)FIELD_GRAB, hinst, NULL);
	SendMessage(keyboardPage.grabBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.flipLabel = CreateWindow(WC_STATIC, "Flip:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.flipLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.flipBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, (HMENU)FIELD_FLIP, hinst, NULL);
	SendMessage(keyboardPage.flipBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.grindLabel = CreateWindow(WC_STATIC, "Grind:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.grindLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.grindBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, (HMENU)FIELD_GRIND, hinst, NULL);
	SendMessage(keyboardPage.grindBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.spinLLabel = CreateWindow(WC_STATIC, "Spin Left:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.spinLLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.spinLBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, (HMENU)FIELD_SPINL, hinst, NULL);
	SendMessage(keyboardPage.spinLBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.spinRLabel = CreateWindow(WC_STATIC, "Spin Right:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 5), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.spinRLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.spinRBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 5), 64, boxHeight, result, (HMENU)FIELD_SPINR, hinst, NULL);
	SendMessage(keyboardPage.spinRBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.nollieLabel = CreateWindow(WC_STATIC, "Nollie:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 6), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.nollieLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.nollieBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 6), 64, boxHeight, result, (HMENU)FIELD_NOLLIE, hinst, NULL);
	SendMessage(keyboardPage.nollieBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.switchLabel = CreateWindow(WC_STATIC, "Switch:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 7), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.switchLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.switchBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 7), 64, boxHeight, result, (HMENU)FIELD_SWITCH, hinst, NULL);
	SendMessage(keyboardPage.switchBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.pauseLabel = CreateWindow(WC_STATIC, "Pause:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 8), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.pauseLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.pauseBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, resolutionGroupY + 16 + (boxVSpacing * 8), 64, boxHeight, result, (HMENU)FIELD_PAUSE, hinst, NULL);
	SendMessage(keyboardPage.pauseBox, WM_SETFONT, (WPARAM)hfont, TRUE);

	SetWindowPos(keyboardPage.actionGroup, keyboardPage.pauseBox, resolutionGroupX, resolutionGroupY, pageWidth - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	// CAMERA BOX
	int graphicsGroupX = (pageWidth / 2) + 8;
	int graphicsGroupY = 8;
	labelHPos = graphicsGroupX + 8;
	boxHPos = graphicsGroupX + ((pageWidth / 2) - 16 - 8 - 64);
	boxVSpacing = 24;

	keyboardPage.movementGroup = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Skater Controls", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, graphicsGroupX, graphicsGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.movementGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	keyboardPage.forwardLabel = CreateWindow(WC_STATIC, "Forward:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16, 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.forwardLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.forwardBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, graphicsGroupY + 16, 64, boxHeight, result, (HMENU)FIELD_FORWARD, hinst, NULL);
	SendMessage(keyboardPage.forwardBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.backwardLabel = CreateWindow(WC_STATIC, "Backward:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.backwardLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.backwardBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, graphicsGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, (HMENU)FIELD_BACKWARD, hinst, NULL);
	SendMessage(keyboardPage.backwardBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.leftLabel = CreateWindow(WC_STATIC, "Left:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.leftLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.leftBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, graphicsGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, (HMENU)FIELD_LEFT, hinst, NULL);
	SendMessage(keyboardPage.leftBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.rightLabel = CreateWindow(WC_STATIC, "Right:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.rightLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.rightBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, graphicsGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, (HMENU)FIELD_RIGHT, hinst, NULL);
	SendMessage(keyboardPage.rightBox, WM_SETFONT, (WPARAM)hfont, TRUE);

	SetWindowPos(keyboardPage.movementGroup, keyboardPage.rightBox, graphicsGroupX, graphicsGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	// MISC BOX
	int miscGroupX = (pageWidth / 2) + 8;
	int miscGroupY = (pageHeight / 2) + 8;
	keyboardPage.cameraGroup = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Camera Controls", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, miscGroupX, miscGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.cameraGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	keyboardPage.camUpLabel = CreateWindow(WC_STATIC, "Camera Up:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16, 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.camUpLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camUpBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, miscGroupY + 16, 64, boxHeight, result, (HMENU)FIELD_CAMUP, hinst, NULL);
	SendMessage(keyboardPage.camUpBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camDownLabel = CreateWindow(WC_STATIC, "Camera Down:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 1), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.camDownLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camDownBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, miscGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, (HMENU)FIELD_CAMDOWN, hinst, NULL);
	SendMessage(keyboardPage.camDownBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camLeftLabel = CreateWindow(WC_STATIC, "Camera Left:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 2), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.camLeftLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camLeftBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, miscGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, (HMENU)FIELD_CAMLEFT, hinst, NULL);
	SendMessage(keyboardPage.camLeftBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camRightLabel = CreateWindow(WC_STATIC, "Camera Right:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 3), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.camRightLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.camRightBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, miscGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, (HMENU)FIELD_CAMRIGHT, hinst, NULL);
	SendMessage(keyboardPage.camRightBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.viewToggleLabel = CreateWindow(WC_STATIC, "View Toggle:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 4), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.viewToggleLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.viewToggleBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, miscGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, (HMENU)FIELD_VIEWTOGGLE, hinst, NULL);
	SendMessage(keyboardPage.viewToggleBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.swivelLockLabel = CreateWindow(WC_STATIC, "Swivel Lock:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 5), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(keyboardPage.swivelLockLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	keyboardPage.swivelLockBox = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, "", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, boxHPos, miscGroupY + 16 + (boxVSpacing * 5), 64, boxHeight, result, (HMENU)FIELD_SWIVELLOCK, hinst, NULL);
	SendMessage(keyboardPage.swivelLockBox, WM_SETFONT, (WPARAM)hfont, TRUE);

	SetWindowPos(keyboardPage.cameraGroup, keyboardPage.swivelLockBox, miscGroupX, miscGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	keyboardPage.page = result;

	return result;
}

#define COMBOBOX_OLLIE 0x8801
#define COMBOBOX_GRAB 0x8802
#define COMBOBOX_FLIP 0x8803
#define COMBOBOX_GRIND 0x8804
#define COMBOBOX_SPINL 0x8805
#define COMBOBOX_SPINR 0x8806
#define COMBOBOX_NOLLIE 0x8807
#define COMBOBOX_SWITCH 0x8808
#define COMBOBOX_PAUSE 0x8809
#define COMBOBOX_FORWARD 0x880a
#define COMBOBOX_BACKWARD 0x880b
#define COMBOBOX_LEFT 0x880c
#define COMBOBOX_RIGHT 0x880d
#define COMBOBOX_MOVESTICK 0x880e
#define COMBOBOX_CAMSTICK 0x880f
#define COMBOBOX_VIEWTOGGLE 0x8810
#define COMBOBOX_SWIVELLOCK 0x8811

setButtonBindBox(HWND box, controllerButton bind) {
	int sel = 0;

	switch(bind) {
	case CONTROLLER_UNBOUND:
		sel = 0;
		break;
	case CONTROLLER_BUTTON_A:
		sel = 1;
		break;
	case CONTROLLER_BUTTON_B:
		sel = 2;
		break;
	case CONTROLLER_BUTTON_X:
		sel = 3;
		break;
	case CONTROLLER_BUTTON_Y:
		sel = 4;
		break;
	case CONTROLLER_BUTTON_DPAD_UP:
		sel = 5;
		break;
	case CONTROLLER_BUTTON_DPAD_DOWN:
		sel = 6;
		break;
	case CONTROLLER_BUTTON_DPAD_LEFT:
		sel = 7;
		break;
	case CONTROLLER_BUTTON_DPAD_RIGHT:
		sel = 8;
		break;
	case CONTROLLER_BUTTON_LEFTSHOULDER:
		sel = 9;
		break;
	case CONTROLLER_BUTTON_RIGHTSHOULDER:
		sel = 10;
		break;
	case CONTROLLER_BUTTON_LEFTTRIGGER:
		sel = 11;
		break;
	case CONTROLLER_BUTTON_RIGHTTRIGGER:
		sel = 12;
		break;
	case CONTROLLER_BUTTON_LEFTSTICK:
		sel = 13;
		break;
	case CONTROLLER_BUTTON_RIGHTSTICK:
		sel = 14;
		break;
	case CONTROLLER_BUTTON_BACK:
		sel = 15;
		break;
	case CONTROLLER_BUTTON_START:
		sel = 16;
		break;
	case CONTROLLER_BUTTON_TOUCHPAD:
		sel = 17;
		break;
	case CONTROLLER_BUTTON_PADDLE1:
		sel = 18;
		break;
	case CONTROLLER_BUTTON_PADDLE2:
		sel = 19;
		break;
	case CONTROLLER_BUTTON_PADDLE3:
		sel = 20;
		break;
	case CONTROLLER_BUTTON_PADDLE4:
		sel = 21;
		break;
	}

	ComboBox_SetCurSel(box, sel);
}

setStickBindBox(HWND box, controllerButton bind) {
	int sel = 0;

	switch(bind) {
	case CONTROLLER_STICK_UNBOUND:
		sel = 0;
		break;
	case CONTROLLER_STICK_LEFT:
		sel = 1;
		break;
	case CONTROLLER_STICK_RIGHT:
		sel = 2;
		break;
	}

	ComboBox_SetCurSel(box, sel);
}

void setAllPadBindText() {
	setButtonBindBox(gamepadPage.ollieBox, padbinds.ollie);
	setButtonBindBox(gamepadPage.grabBox, padbinds.grab);
	setButtonBindBox(gamepadPage.flipBox, padbinds.kick);
	setButtonBindBox(gamepadPage.grindBox, padbinds.grind);
	setButtonBindBox(gamepadPage.spinLBox, padbinds.leftSpin);
	setButtonBindBox(gamepadPage.spinRBox, padbinds.rightSpin);
	setButtonBindBox(gamepadPage.nollieBox, padbinds.nollie);
	setButtonBindBox(gamepadPage.switchBox, padbinds.switchRevert);
	setButtonBindBox(gamepadPage.pauseBox, padbinds.menu);

	setButtonBindBox(gamepadPage.forwardBox, padbinds.up);
	setButtonBindBox(gamepadPage.backwardBox, padbinds.down);
	setButtonBindBox(gamepadPage.leftBox, padbinds.left);
	setButtonBindBox(gamepadPage.rightBox, padbinds.right);
	setStickBindBox(gamepadPage.moveStickBox, padbinds.movement);

	setStickBindBox(gamepadPage.camStickBox, padbinds.camera);
	setButtonBindBox(gamepadPage.viewToggleBox, padbinds.cameraToggle);
	setButtonBindBox(gamepadPage.swivelLockBox, padbinds.cameraSwivelLock);
}

controllerButton buttonLUT[] = {
	CONTROLLER_UNBOUND,
	CONTROLLER_BUTTON_A,
	CONTROLLER_BUTTON_B,
	CONTROLLER_BUTTON_X,
	CONTROLLER_BUTTON_Y,
	CONTROLLER_BUTTON_DPAD_UP,
	CONTROLLER_BUTTON_DPAD_DOWN,
	CONTROLLER_BUTTON_DPAD_LEFT,
	CONTROLLER_BUTTON_DPAD_RIGHT,
	CONTROLLER_BUTTON_LEFTSHOULDER,
	CONTROLLER_BUTTON_RIGHTSHOULDER,
	CONTROLLER_BUTTON_LEFTTRIGGER,
	CONTROLLER_BUTTON_RIGHTTRIGGER,
	CONTROLLER_BUTTON_LEFTSTICK,
	CONTROLLER_BUTTON_RIGHTSTICK,
	CONTROLLER_BUTTON_BACK,
	CONTROLLER_BUTTON_START,
	CONTROLLER_BUTTON_TOUCHPAD,
	CONTROLLER_BUTTON_PADDLE1,
	CONTROLLER_BUTTON_PADDLE2,
	CONTROLLER_BUTTON_PADDLE3,
	CONTROLLER_BUTTON_PADDLE4,
};

controllerButton stickLUT[] = {
	CONTROLLER_STICK_UNBOUND,
	CONTROLLER_STICK_LEFT,
	CONTROLLER_STICK_RIGHT,
};

LRESULT CALLBACK gamepadPageProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_COMMAND: {
			int controlId = LOWORD(wParam);
			int controlCode = HIWORD(wParam);

			if (controlCode == 1) {
				int idx = ComboBox_GetCurSel(lParam);

				switch (controlId) {
					case COMBOBOX_OLLIE:
						padbinds.ollie = buttonLUT[idx];
						break;
					case COMBOBOX_GRAB:
						padbinds.grab = buttonLUT[idx];
						break;
					case COMBOBOX_FLIP:
						padbinds.kick = buttonLUT[idx];
						break;
					case COMBOBOX_GRIND:
						padbinds.grind = buttonLUT[idx];
						break;
					case COMBOBOX_SPINL:
						padbinds.leftSpin = buttonLUT[idx];
						break;
					case COMBOBOX_SPINR:
						padbinds.rightSpin = buttonLUT[idx];
						break;
					case COMBOBOX_NOLLIE:
						padbinds.nollie = buttonLUT[idx];
						break;
					case COMBOBOX_SWITCH:
						padbinds.switchRevert = buttonLUT[idx];
						break;
					case COMBOBOX_PAUSE:
						padbinds.menu = buttonLUT[idx];
						break;
					case COMBOBOX_FORWARD:
						padbinds.up = buttonLUT[idx];
						break;
					case COMBOBOX_BACKWARD:
						padbinds.down = buttonLUT[idx];
						break;
					case COMBOBOX_LEFT:
						padbinds.left = buttonLUT[idx];
						break;
					case COMBOBOX_RIGHT:
						padbinds.right = buttonLUT[idx];
						break;
					case COMBOBOX_MOVESTICK:
						padbinds.movement = stickLUT[idx];
						break;
					case COMBOBOX_CAMSTICK:
						padbinds.camera = stickLUT[idx];
						break;
					case COMBOBOX_VIEWTOGGLE:
						padbinds.cameraToggle = buttonLUT[idx];
						break;
					case COMBOBOX_SWIVELLOCK:
						padbinds.cameraSwivelLock = buttonLUT[idx];
						break;
					default:
						break;
				}
			}
		

			return 0;
		}
	}


	return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

HWND createButtonCombobox(int x, int y, int w, int h, HWND parent, int id, HINSTANCE hinst) {
	HWND result = CreateWindow(WC_COMBOBOX, "", WS_TABSTOP | WS_VISIBLE | WS_CHILD | CBS_DROPDOWNLIST, x, y, w, h, parent, id, hinst, NULL);
	SendMessage(result, WM_SETFONT, (WPARAM)hfont, TRUE);

	ComboBox_AddString(result, "Unbound");
	ComboBox_AddString(result, "A");
	ComboBox_AddString(result, "B");
	ComboBox_AddString(result, "X");
	ComboBox_AddString(result, "Y");
	ComboBox_AddString(result, "D-Pad Up");
	ComboBox_AddString(result, "D-Pad Down");
	ComboBox_AddString(result, "D-Pad Left");
	ComboBox_AddString(result, "D-Pad Right");
	ComboBox_AddString(result, "L1/Left Bumper");
	ComboBox_AddString(result, "R1/Right Bumper");
	ComboBox_AddString(result, "L2/Left Trigger");
	ComboBox_AddString(result, "R2/Right Trigger");
	ComboBox_AddString(result, "L3/Left Stick");
	ComboBox_AddString(result, "R3/Right Stick");
	ComboBox_AddString(result, "Select/Back");
	ComboBox_AddString(result, "Start/Options");
	ComboBox_AddString(result, "Touchpad");
	ComboBox_AddString(result, "Paddle 1");
	ComboBox_AddString(result, "Paddle 2");
	ComboBox_AddString(result, "Paddle 3");
	ComboBox_AddString(result, "Paddle 4");

	ComboBox_SetCurSel(result, 0);

	return result;
}

HWND createStickCombobox(int x, int y, int w, int h, HWND parent, HMENU menu, HINSTANCE hinst) {
	HWND result = CreateWindow(WC_COMBOBOX, "", WS_TABSTOP | WS_VISIBLE | WS_CHILD | CBS_DROPDOWNLIST, x, y, w, h, parent, menu, hinst, NULL);
	SendMessage(result, WM_SETFONT, (WPARAM)hfont, TRUE);

	ComboBox_AddString(result, "Unbound");
	ComboBox_AddString(result, "Left Stick");
	ComboBox_AddString(result, "Right Stick");

	ComboBox_SetCurSel(result, 0);

	return result;
}

HWND createGamepadPage(HWND parent, HINSTANCE hinst, RECT rect) {
	// TODO: create struct here
	//HWND result = CreateWindow(WC_STATIC, L"", WS_CHILD | WS_VISIBLE, rect.left, rect.top, rect.right, rect.bottom, parent, NULL, hinst, NULL);

	const char *className  = "Gamepad Page Class";

	WNDCLASS wc = { 0,
		gamepadPageProc,
		0,
		0,
		hinst,
		NULL,
		NULL,
		NULL,
		NULL,
		className
	};

	RegisterClass(&wc);

	HWND result = CreateWindow(className, L"", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, parent, NULL, hinst, NULL);
	// TODO: add handler that destroys struct

	RECT pageRect;
	GetClientRect(result, &pageRect);

	int pageWidth = pageRect.right - pageRect.left;
	int pageHeight = pageRect.bottom - pageRect.top;

	// SKATER BOX
	int resolutionGroupX = 8;
	int resolutionGroupY = 8;
	int boxHeight = 20;
	int labelHPos = resolutionGroupX + 8;
	int boxHPos = resolutionGroupX + ((pageWidth / 2) - 16 - 8 - 64);
	int boxVSpacing = 32;

	gamepadPage.actionGroup = CreateWindow(WC_BUTTON, "Actions", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, resolutionGroupX, resolutionGroupY, (pageWidth / 2) - 16, pageHeight - 16, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.actionGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	gamepadPage.ollieLabel = CreateWindow(WC_STATIC, "Ollie:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16, 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.ollieLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.ollieBox = createButtonCombobox(boxHPos, resolutionGroupY + 16, 64, boxHeight, result, (HMENU)COMBOBOX_OLLIE, hinst);
	gamepadPage.grabLabel = CreateWindow(WC_STATIC, "Grab:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.grabLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.grabBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, (HMENU)COMBOBOX_GRAB, hinst);
	gamepadPage.flipLabel = CreateWindow(WC_STATIC, "Flip:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.flipLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.flipBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, (HMENU)COMBOBOX_FLIP, hinst);
	gamepadPage.grindLabel = CreateWindow(WC_STATIC, "Grind:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.grindLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.grindBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, (HMENU)COMBOBOX_GRIND, hinst);
	gamepadPage.spinLLabel = CreateWindow(WC_STATIC, "Spin Left:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.spinLLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.spinLBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, (HMENU)COMBOBOX_SPINL, hinst);
	gamepadPage.spinRLabel = CreateWindow(WC_STATIC, "Spin Right:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 5), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.spinRLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.spinRBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 5), 64, boxHeight, result, (HMENU)COMBOBOX_SPINR, hinst);
	gamepadPage.nollieLabel = CreateWindow(WC_STATIC, "Nollie:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 6), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.nollieLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.nollieBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 6), 64, boxHeight, result, (HMENU)COMBOBOX_NOLLIE, hinst);
	gamepadPage.switchLabel = CreateWindow(WC_STATIC, "Switch:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 7), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.switchLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.switchBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 7), 64, boxHeight, result, (HMENU)COMBOBOX_SWITCH, hinst);
	SendMessage(gamepadPage.switchBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.pauseLabel = CreateWindow(WC_STATIC, "Pause:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, resolutionGroupY + 16 + (boxVSpacing * 8), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.pauseLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.pauseBox = createButtonCombobox(boxHPos, resolutionGroupY + 16 + (boxVSpacing * 8), 64, boxHeight, result, (HMENU)COMBOBOX_PAUSE, hinst);

	SetWindowPos(gamepadPage.actionGroup, gamepadPage.pauseBox, resolutionGroupX, resolutionGroupY, pageWidth - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	// CAMERA BOX
	int graphicsGroupX = (pageWidth / 2) + 8;
	int graphicsGroupY = 8;
	labelHPos = graphicsGroupX + 8;
	boxHPos = graphicsGroupX + ((pageWidth / 2) - 16 - 8 - 64);
	boxVSpacing = 24;

	gamepadPage.movementGroup = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Skater Controls", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, graphicsGroupX, graphicsGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.movementGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	gamepadPage.forwardLabel = CreateWindow(WC_STATIC, "Forward:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16, 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.forwardLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.forwardBox = createButtonCombobox(boxHPos, graphicsGroupY + 16, 64, boxHeight, result, (HMENU)COMBOBOX_FORWARD, hinst);
	gamepadPage.backwardLabel = CreateWindow(WC_STATIC, "Backward:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.backwardLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.backwardBox = createButtonCombobox(boxHPos, graphicsGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, (HMENU)COMBOBOX_BACKWARD, hinst);
	gamepadPage.leftLabel = CreateWindow(WC_STATIC, "Left:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.leftLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.leftBox = createButtonCombobox(boxHPos, graphicsGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, (HMENU)COMBOBOX_LEFT, hinst);
	gamepadPage.rightLabel = CreateWindow(WC_STATIC, "Right:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.rightLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.rightBox = createButtonCombobox(boxHPos, graphicsGroupY + 16 + (boxVSpacing * 3), 64, boxHeight, result, (HMENU)COMBOBOX_RIGHT, hinst);
	gamepadPage.moveStickLabel = CreateWindow(WC_STATIC, "Move Stick:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, graphicsGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.moveStickLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.moveStickBox = createStickCombobox(boxHPos, graphicsGroupY + 16 + (boxVSpacing * 4), 64, boxHeight, result, (HMENU)COMBOBOX_MOVESTICK, hinst);

	SetWindowPos(gamepadPage.movementGroup, gamepadPage.moveStickBox, graphicsGroupX, graphicsGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	// MISC BOX
	int miscGroupX = (pageWidth / 2) + 8;
	int miscGroupY = (pageHeight / 2) + 8;
	gamepadPage.cameraGroup = CreateWindowEx(WS_EX_CONTROLPARENT, WC_BUTTON, "Camera Controls", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, miscGroupX, miscGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.cameraGroup, WM_SETFONT, (WPARAM)hfont, TRUE);

	gamepadPage.camStickLabel = CreateWindow(WC_STATIC, "Camera Stick:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16, 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.camStickLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.camStickBox = createStickCombobox(boxHPos, miscGroupY + 16, 64, boxHeight, result, (HMENU)COMBOBOX_CAMSTICK, hinst);
	SendMessage(gamepadPage.camStickBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.viewToggleLabel = CreateWindow(WC_STATIC, "View Toggle:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 1), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.viewToggleLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.viewToggleBox = createButtonCombobox(boxHPos, miscGroupY + 16 + (boxVSpacing * 1), 64, boxHeight, result, (HMENU)COMBOBOX_VIEWTOGGLE, hinst);
	SendMessage(gamepadPage.viewToggleBox, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.swivelLockLabel = CreateWindow(WC_STATIC, "Swivel Lock:", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, labelHPos, miscGroupY + 16 + (boxVSpacing * 2), 80, boxHeight, result, NULL, hinst, NULL);
	SendMessage(gamepadPage.swivelLockLabel, WM_SETFONT, (WPARAM)hfont, TRUE);
	gamepadPage.swivelLockBox = createButtonCombobox(boxHPos, miscGroupY + 16 + (boxVSpacing * 2), 64, boxHeight, result, (HMENU)COMBOBOX_SWIVELLOCK, hinst);
	SendMessage(gamepadPage.swivelLockBox, WM_SETFONT, (WPARAM)hfont, TRUE);

	SetWindowPos(gamepadPage.cameraGroup, gamepadPage.swivelLockBox, miscGroupX, miscGroupY, (pageWidth / 2) - 16, (pageHeight / 2) - 16, HWND_BOTTOM);

	gamepadPage.page = result;

	return result;
}

HWND createTabs(HWND parent, HINSTANCE hinst) {
	RECT wndRect;
	HWND result;
	
	GetClientRect(parent, &wndRect);

	wndRect.left += 8;
	wndRect.right -= 8;
	wndRect.top += 8 + 96;
	wndRect.bottom -= 8 + 32;

	result = CreateWindow(WC_TABCONTROL, L"", WS_CHILD | WS_CLIPSIBLINGS | WS_VISIBLE, wndRect.left, wndRect.top, wndRect.right - wndRect.left, wndRect.bottom - wndRect.top, parent, NULL, hinst, NULL);
	SendMessage(result, WM_SETFONT, (WPARAM)hfont, TRUE);

	TCITEM item;
	item.mask = TCIF_TEXT | TCIF_IMAGE;
	item.iImage = -1;
	
	item.pszText = "General";
	TabCtrl_InsertItem(result, 0, &item);

	item.pszText = "Keyboard";
	TabCtrl_InsertItem(result, 1, &item);

	item.pszText = "Gamepad";
	TabCtrl_InsertItem(result, 2, &item);

	RECT tabRect;

	GetClientRect(result, &tabRect);

	TabCtrl_AdjustRect(result, FALSE, &tabRect);

	createSettingsPage(result, hinst, tabRect);
	createKeyboardPage(result, hinst, tabRect);
	createGamepadPage(result, hinst, tabRect);

	SetWindowPos(result, gamepadPage.page, wndRect.left, wndRect.top, wndRect.right - wndRect.left, wndRect.bottom - wndRect.top, HWND_BOTTOM);

	return result;
}

#define BUTTON_OK 0x8801
#define BUTTON_CANCEL 0x8802
#define BUTTON_DEFAULTS 0x8803

HWND createButtons(HWND parent, HINSTANCE hinst) {
	HWND result;

	RECT wndRect;
	GetClientRect(parent, &wndRect);

	wndRect.left += 8;
	wndRect.right -= 8;
	wndRect.top = wndRect.bottom - 42 + 8;
	wndRect.bottom -= 8;

	HWND restoreDefaultButton = CreateWindow(WC_BUTTON, "Restore Defaults", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, wndRect.left, wndRect.top, 96, wndRect.bottom - wndRect.top, parent, BUTTON_DEFAULTS, hinst, NULL);
	SendMessage(restoreDefaultButton, WM_SETFONT, (WPARAM)hfont, TRUE);
	HWND cancelButton = CreateWindow(WC_BUTTON, "Cancel", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, wndRect.right - (80 * 2) - 8, wndRect.top, 80, wndRect.bottom - wndRect.top, parent, BUTTON_CANCEL, hinst, NULL);
	SendMessage(cancelButton, WM_SETFONT, (WPARAM)hfont, TRUE);
	HWND okButton = CreateWindow(WC_BUTTON, "OK", WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, wndRect.right - 80, wndRect.top, 80, wndRect.bottom - wndRect.top, parent, BUTTON_OK, hinst, NULL);
	SendMessage(okButton, WM_SETFONT, (WPARAM)hfont, TRUE);
}

LOGBRUSH lbrBk;

LRESULT CALLBACK wndProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	
	switch (uMsg)
	{
	case WM_DESTROY:
		PostQuitMessage(0);
		return 0;

	case WM_PAINT:
		{
			PAINTSTRUCT ps;
			HDC hdc = BeginPaint(hwnd, &ps);

			// All painting occurs here, between BeginPaint and EndPaint.

			FillRect(hdc, &ps.rcPaint, (HBRUSH) (COLOR_WINDOW));

			EndPaint(hwnd, &ps);
		}
		return 0;

	case WM_NOTIFY: {
		//printf("NOTIFY!!! %x\n", ((LPNMHDR)lParam)->hwndFrom);
		int code = ((LPNMHDR)lParam)->code;

		if (code == TCN_SELCHANGE) {
			//printf("WOW!!\n");
			int tab = TabCtrl_GetCurSel(((LPNMHDR)lParam)->hwndFrom);
			//printf("SELECTED TAB: %d\n", tab);
			if (tab == 0) {
				ShowWindow(settingsPage.page, SW_SHOW);
				ShowWindow(keyboardPage.page, SW_HIDE);
				ShowWindow(gamepadPage.page, SW_HIDE);
			} else if (tab == 1) {
				ShowWindow(settingsPage.page, SW_HIDE);
				ShowWindow(keyboardPage.page, SW_SHOW);
				ShowWindow(gamepadPage.page, SW_HIDE);
			} else {
				ShowWindow(settingsPage.page, SW_HIDE);
				ShowWindow(keyboardPage.page, SW_HIDE);
				ShowWindow(gamepadPage.page, SW_SHOW);
			}


		}

		return 0;
	}
	case WM_COMMAND:
		printf("COMMAND CHILD!!! LO: %d, HI: %d, L: %d\n", LOWORD(wParam), HIWORD(wParam), lParam);
		int controlId = LOWORD(wParam);

		if (controlId == BUTTON_OK) {
			saveSettings();
			PostQuitMessage(0);
		} else if (controlId == BUTTON_CANCEL) {
			PostQuitMessage(0);
		} else if (controlId == BUTTON_DEFAULTS) {
			defaultSettings();
			updateSettingsPage();
			setAllBindText();
			setAllPadBindText();
		}

		return 0;
	}

	return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

int main(int argc, char **argv) {
	HINSTANCE hinst = GetModuleHandle(NULL);

	INITCOMMONCONTROLSEX icex;
	icex.dwSize = sizeof(INITCOMMONCONTROLSEX);
	icex.dwICC = ICC_TAB_CLASSES;
	InitCommonControlsEx(&icex);

	DWORD dwDlgBase = GetDialogBaseUnits();
	cxMargin = LOWORD(dwDlgBase) / 4; 
	cyMargin = HIWORD(dwDlgBase) / 8; 

	LOGFONT lf;
	GetObject (GetStockObject(DEFAULT_GUI_FONT), sizeof(LOGFONT), &lf); 
	hfont = CreateFont (lf.lfHeight, lf.lfWidth, 
		lf.lfEscapement, lf.lfOrientation, lf.lfWeight, 
		lf.lfItalic, lf.lfUnderline, lf.lfStrikeOut, lf.lfCharSet, 
		lf.lfOutPrecision, lf.lfClipPrecision, lf.lfQuality, 
		lf.lfPitchAndFamily, lf.lfFaceName); 

	const wchar_t CLASS_NAME[]  = L"Config Window Class";

	WNDCLASS wc = { 0,
		wndProc,
		0,
		0,
		hinst,
		NULL,
		NULL,
		NULL,
		NULL,
		CLASS_NAME
	};

	RegisterClass(&wc);

	windowHwnd = CreateWindow(CLASS_NAME, "PARTYMOD Configuration", WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX, CW_USEDEFAULT, CW_USEDEFAULT, 400, 550, NULL, NULL, hinst, NULL);
	SendMessage(windowHwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	initResolutionList();

	createTabs(windowHwnd, hinst);

	createButtons(windowHwnd, hinst);

	loadSettings();
	updateSettingsPage();
	cursedSDLSetup();
	setAllBindText();
	setAllPadBindText();

	ShowWindow(windowHwnd, SW_NORMAL);

	// Run the message loop.

	MSG msg;
	while (GetMessage(&msg, NULL, 0, 0) > 0)
	{
		TranslateMessage(&msg);
		DispatchMessage(&msg);
	}

	return 0;
}
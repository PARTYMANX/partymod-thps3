#include <config.h>

#include <windows.h>

#include <stdio.h>
#include <stdint.h>

#include <patch.h>
#include <input.h>
#include <global.h>

#include <SDL2/SDL.h>
#include <SDL2/SDL_syswm.h>

int *resX = 0x00851084;
int *resY = 0x00851088;
int *bitDepth = 0x0085108c;

uint8_t *highBandwidth = 0x005b4e75;
uint8_t *shadows = 0x005b4e76;
uint8_t *particles = 0x005b4e77;
uint8_t *animatingTextures = 0x005b4e78;
uint8_t *playIntro = 0x005b4e79;
uint8_t *customSettings = 0x008510b1;
uint8_t *distanceFog = 0x008510b2;
uint8_t *lowDetailModels = 0x008510b3;
uint8_t *frameCap = 0x008510b4;

uint8_t *isWindowed = 0x008510a9;
HWND *windowHandle = 0x0085109c;

float *aspectRatio1 = 0x0058eb14;
float *aspectRatio2 = 0x0058d96c;

uint8_t borderless;

SDL_Window *window;

void dumpSettings() {
	printf("RESOLUTION X: %d\n", *resX);
	printf("RESOLUTION Y: %d\n", *resY);
	printf("BIT DEPTH: %d\n", *bitDepth);
	printf("HIGH BANDWIDTH: %02x\n", *highBandwidth);
	printf("PLAY INTRO: %02x\n", *playIntro);
	printf("CUSTOM SETTINGS: %02x\n", *customSettings);
	printf("ANIMATING TEXTURES: %02x\n", *animatingTextures);
	printf("SHADOWS: %02x\n", *shadows);
	printf("PARTICLES: %02x\n", *particles);
	printf("DISTANCE FOG: %02x\n", *distanceFog);
	printf("LOW DETAIL MODELS: %02x\n", *lowDetailModels);
	printf("LOCK TO 60HZ: %02x\n", *frameCap);
}

void enforceMaxResolution() {
	DEVMODE deviceMode;
	int i = 0;
	uint8_t isValidX = 0;
	uint8_t isValidY = 0;

	while (EnumDisplaySettings(NULL, i, &deviceMode)) {
		if (deviceMode.dmPelsWidth >= *resX) {
			isValidX = 1;
		}
		if (deviceMode.dmPelsHeight >= *resY) {
			isValidY = 1;
		}

		i++;
	}

	if (!isValidX || !isValidY) {
		*resX = 0;
		*resY = 0;
	}
}

HWND getOrCreateWindow() {
	if (!(*windowHandle)) {
		SDL_Init(SDL_INIT_VIDEO);

		SDL_WindowFlags flags = *isWindowed ? SDL_WINDOW_SHOWN : SDL_WINDOW_FULLSCREEN;

		if (borderless && *isWindowed) {
			flags |= SDL_WINDOW_BORDERLESS;
		}

		enforceMaxResolution();

		if (*resX == 0 || *resY == 0) {
			SDL_DisplayMode displayMode;
			SDL_GetDesktopDisplayMode(0, &displayMode);
			*resX = displayMode.w;
			*resY = displayMode.h;
		}
		
		if (*resX < 640) {
			*resX = 640;
		}
		if (*resY < 480) {
			*resY = 480;
		}

		window = SDL_CreateWindow("THPS3 - PARTYMOD", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, *resX, *resY, flags);   // TODO: fullscreen

		if (!window) {
			printf("Failed to create window! Error: %s\n", SDL_GetError());
		}

		SDL_SysWMinfo wmInfo;
		SDL_VERSION(&wmInfo.version);
		SDL_GetWindowWMInfo(window, &wmInfo);
		*windowHandle = wmInfo.info.win.window;
	}

	return *windowHandle;
}

void patchWindow() {
	// replace the window with an SDL2 window.  this kind of straddles the line between input and config
	//DWORD style = WS_CAPTION | WS_ICONIC | WS_CLIPCHILDREN | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE | WS_BORDER;

	//patchDWord((void *)0x00409d55, style);
	//patchDWord((void *)0x00409df4, style);
	patchCall((void *)0x00409be0, getOrCreateWindow);
	patchByte((void *)(0x00409be0 + 5), 0xC3);  // RET

	// always say the game is windowed
	patchNop((void *)0x00409f70, 10);
	patchInst((void *)0x00409f70, MOVIMM8);
	patchByte((void *)(0x00409f70 + 1), 0x01);

	// make the game hide the cursor when it thinks its windowed
	patchByte((void *)0x004075e5, 0x74);    // JNZ -> JZ
	patchByte((void *)0x0040773b, 0x74);    // JNZ -> JZ
	patchByte((void *)0x00425f42, 0x74);    // JNZ -> JZ
	patchByte((void *)0x00449e6f, 0x74);    // JNZ -> JZ

	// remove sleep from main loop
	patchNop((void *)0x004c05eb, 2);
	patchNop((void *)0x004c067a, 8);
	//patchByte((void *)(0x004c067a + 1), 0x01);
}

#define GRAPHICS_SECTION "Graphics"

int getIniBool(char *section, char *key, int def, char *file) {
	int result = GetPrivateProfileInt(section, key, def, file);
	if (result) {
		return 1;
	} else {
		return 0;
	}
}

void loadSettings() {
	printf("Loading settings\n");

	char configFile[1024];
	sprintf(configFile, "%s%s", executableDirectory, CONFIG_FILE_NAME);

	//GetPrivateProfileString("Game", "HighBandwidth", "", )
	*highBandwidth = 1;	// it's 2022, very few people are probably on dial up, and probably generally not gaming
	*playIntro = getIniBool("Miscellaneous", "PlayIntro", 1, configFile);

	*animatingTextures = getIniBool("Graphics", "AnimatedTextures", 1, configFile);
	*particles = getIniBool("Graphics", "Particles", 1, configFile);
	*shadows = getIniBool("Graphics", "Shadows", 1, configFile);
	*distanceFog = getIniBool("Graphics", "DistanceFog", 0, configFile);
	*lowDetailModels = getIniBool("Graphics", "LowDetailModels", 0, configFile);
   
	*resX = GetPrivateProfileInt("Graphics", "ResolutionX", 640, configFile);
	*resY = GetPrivateProfileInt("Graphics", "ResolutionY", 480, configFile);
	if (!*isWindowed) {
		*isWindowed = getIniBool("Graphics", "Windowed", 0, configFile);
	}
	borderless = getIniBool("Graphics", "Borderless", 0, configFile);

	*customSettings = 1;    // not useful
	*frameCap = 1;  // not useful...maybe
	*bitDepth = 32;	// forcing this to 32 as it's 2022

	float aspectRatio = ((float)*resX) / ((float)*resY);
	patchFloat(0x0058eb14, aspectRatio);
	patchFloat(0x0058d96c, aspectRatio);

	// shadows - i'm not fully sure how this works, but in theory you can increase shadow resolution here
	//patchFloat(0x00501289 + 4, 100.0f);
	//patchFloat(0x00501291 + 4, 100.0f);
	//patchFloat(0x005012a4 + 4, 128.0f);
	//patchFloat(0x005012ac + 4, 128.0f);
}

//char *keyFmtStr = "menu=%d\0cameraToggle=%d\0cameraSwivelLock=%d\0grind=%d\0grab=%d\0ollie=%d\0kick=%d\0spinLeft=%d\0spinRight=%d\0nollie=%d\0switch=%d\0left=%d\0right=%d\0up=%d\0down=%d\0"

#define KEYBIND_SECTION "Keybinds"
#define CONTROLLER_SECTION "Gamepad"

void loadKeyBinds(struct keybinds *bindsOut, uint8_t *usingHardCodedControls) {
	char configFile[1024];
	sprintf(configFile, "%s%s", executableDirectory, CONFIG_FILE_NAME);

	if (bindsOut) {
		bindsOut->menu = GetPrivateProfileInt(KEYBIND_SECTION, "Pause", 0, configFile);
		bindsOut->cameraToggle = GetPrivateProfileInt(KEYBIND_SECTION, "ViewToggle", SDL_SCANCODE_GRAVE, configFile);
		bindsOut->cameraSwivelLock = GetPrivateProfileInt(KEYBIND_SECTION, "SwivelLock", 0, configFile);

		bindsOut->grind = GetPrivateProfileInt(KEYBIND_SECTION, "Grind", SDL_SCANCODE_KP_8, configFile);
		bindsOut->grab = GetPrivateProfileInt(KEYBIND_SECTION, "Grab", SDL_SCANCODE_KP_6, configFile);
		bindsOut->ollie = GetPrivateProfileInt(KEYBIND_SECTION, "Ollie", SDL_SCANCODE_KP_2, configFile);
		bindsOut->kick = GetPrivateProfileInt(KEYBIND_SECTION, "Flip", SDL_SCANCODE_KP_4, configFile);

		bindsOut->leftSpin = GetPrivateProfileInt(KEYBIND_SECTION, "SpinLeft", SDL_SCANCODE_KP_1, configFile);
		bindsOut->rightSpin = GetPrivateProfileInt(KEYBIND_SECTION, "SpinRight", SDL_SCANCODE_KP_3, configFile);
		bindsOut->nollie = GetPrivateProfileInt(KEYBIND_SECTION, "Nollie", SDL_SCANCODE_KP_7, configFile);
		bindsOut->switchRevert = GetPrivateProfileInt(KEYBIND_SECTION, "Switch", SDL_SCANCODE_KP_9, configFile);

		bindsOut->right = GetPrivateProfileInt(KEYBIND_SECTION, "Right", SDL_SCANCODE_D, configFile);
		bindsOut->left = GetPrivateProfileInt(KEYBIND_SECTION, "Left", SDL_SCANCODE_A, configFile);
		bindsOut->up = GetPrivateProfileInt(KEYBIND_SECTION, "Forward", SDL_SCANCODE_W, configFile);
		bindsOut->down = GetPrivateProfileInt(KEYBIND_SECTION, "Backward", SDL_SCANCODE_S, configFile);

		bindsOut->cameraRight = GetPrivateProfileInt(KEYBIND_SECTION, "CameraRight", SDL_SCANCODE_L, configFile);
		bindsOut->cameraLeft = GetPrivateProfileInt(KEYBIND_SECTION, "CameraLeft", SDL_SCANCODE_J, configFile);
		bindsOut->cameraUp = GetPrivateProfileInt(KEYBIND_SECTION, "CameraUp", SDL_SCANCODE_I, configFile);
		bindsOut->cameraDown = GetPrivateProfileInt(KEYBIND_SECTION, "CameraDown", SDL_SCANCODE_K, configFile);
	}

	if (usingHardCodedControls) {
		*usingHardCodedControls = getIniBool("Miscellaneous", "UseHardCodedControls", 1, configFile);
	}
}

void loadControllerBinds(struct controllerbinds *bindsOut) {
	char configFile[1024];
	sprintf(configFile, "%s%s", executableDirectory, CONFIG_FILE_NAME);

	if (bindsOut) {
		bindsOut->menu = GetPrivateProfileInt(CONTROLLER_SECTION, "Pause", CONTROLLER_BUTTON_START, configFile);
		bindsOut->cameraToggle = GetPrivateProfileInt(CONTROLLER_SECTION, "ViewToggle", CONTROLLER_BUTTON_BACK, configFile);
		bindsOut->cameraSwivelLock = GetPrivateProfileInt(CONTROLLER_SECTION, "SwivelLock", CONTROLLER_BUTTON_RIGHTSTICK, configFile);

		bindsOut->grind = GetPrivateProfileInt(CONTROLLER_SECTION, "Grind", CONTROLLER_BUTTON_Y, configFile);
		bindsOut->grab = GetPrivateProfileInt(CONTROLLER_SECTION, "Grab", CONTROLLER_BUTTON_B, configFile);
		bindsOut->ollie = GetPrivateProfileInt(CONTROLLER_SECTION, "Ollie", CONTROLLER_BUTTON_A, configFile);
		bindsOut->kick = GetPrivateProfileInt(CONTROLLER_SECTION, "Flip", CONTROLLER_BUTTON_X, configFile);

		bindsOut->leftSpin = GetPrivateProfileInt(CONTROLLER_SECTION, "SpinLeft", CONTROLLER_BUTTON_LEFTSHOULDER, configFile);
		bindsOut->rightSpin = GetPrivateProfileInt(CONTROLLER_SECTION, "SpinRight", CONTROLLER_BUTTON_RIGHTSHOULDER, configFile);
		bindsOut->nollie = GetPrivateProfileInt(CONTROLLER_SECTION, "Nollie", CONTROLLER_BUTTON_LEFTTRIGGER, configFile);
		bindsOut->switchRevert = GetPrivateProfileInt(CONTROLLER_SECTION, "Switch", CONTROLLER_BUTTON_RIGHTTRIGGER, configFile);

		bindsOut->right = GetPrivateProfileInt(CONTROLLER_SECTION, "Right", CONTROLLER_BUTTON_DPAD_RIGHT, configFile);
		bindsOut->left = GetPrivateProfileInt(CONTROLLER_SECTION, "Left", CONTROLLER_BUTTON_DPAD_LEFT, configFile);
		bindsOut->up = GetPrivateProfileInt(CONTROLLER_SECTION, "Forward", CONTROLLER_BUTTON_DPAD_UP, configFile);
		bindsOut->down = GetPrivateProfileInt(CONTROLLER_SECTION, "Backward", CONTROLLER_BUTTON_DPAD_DOWN, configFile);

		bindsOut->movement = GetPrivateProfileInt(CONTROLLER_SECTION, "MovementStick", CONTROLLER_STICK_LEFT, configFile);
		bindsOut->camera = GetPrivateProfileInt(CONTROLLER_SECTION, "CameraStick", CONTROLLER_STICK_RIGHT, configFile);
	}
}

void patchLoadConfig() {
	patchCall((void *)0x0040b9f7, loadSettings);
}
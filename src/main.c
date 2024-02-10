#include <windows.h>

#include <stdio.h>
#include <stdint.h>
#include <time.h>

#include <SDL2/SDL.h>

#include <patch.h>
#include <global.h>
#include <input.h>
#include <config.h>
#include <script.h>

#define VERSION_NUMBER_MAJOR 1
#define VERSION_NUMBER_MINOR 0
#define VERSION_NUMBER_PATCH 1

int __stdcall playIntroMovie(char *filename, int unk) {
	void *(__stdcall *getMoviePlayer)(int) = (void *)0x00406d40;
	void (__stdcall *destroyMoviePlayer)() = (void *)0x00406d90;
	void (__stdcall *loadMovie)(char *) = (void *)0x00407940;
	uint64_t (__stdcall *getCurrentTime)() = (void *)0x00409ae0;
	uint8_t (__fastcall *isMovieFinished)(void *) = (void *)0x004077a0;
	void (__fastcall *endMovie)(void *) = (void *)0x00407620;

	void *moviePlayer = getMoviePlayer(0);
	loadMovie(filename);
	uint64_t startTime = getCurrentTime();
	while (1) {
		if (isMovieFinished(moviePlayer)) {
			break;
		}
		uint64_t currentTime = getCurrentTime();
		uint64_t timeElapsed = currentTime - startTime;
		uint8_t isRolled = currentTime < startTime;
		uint64_t elapsedUnk = (currentTime >> 0x20) - (startTime >> 0x20);

		int isSkip = processIntroEvent();
		if (isSkip == 2) {
			endMovie(moviePlayer);
			destroyMoviePlayer();
			return 0;
		}
		if (unk == 2 && (elapsedUnk != isRolled || 250 < timeElapsed) && isSkip) {	// allow skips after a quarter of a second	(original was 3 seconds)
			break;
		}
	}

	endMovie(moviePlayer);
	destroyMoviePlayer();
	return 1;
}

void patchIntroMovie() {
	patchNop((void *)0x0040a1a0, 0x0040a33f - 0x0040a1a0 + 1);
	patchDWord(0x0040a1a0, 0x0824448b);	// MOV EAX, dword ptr [ESP + 0x08] (push second param back onto stack)
	patchByte(0x0040a1a0 + 4, 0x50);		// PUSH EAX
	patchDWord(0x0040a1a0 + 5, 0x0824448b);	// MOV EAX, dword ptr [ESP + 0x08] (push first param back onto stack)
	patchByte(0x0040a1a0 + 9, 0x50);		// PUSH EAX
	patchCall(0x0040a1a0 + 10, playIntroMovie);
	patchDWord(0x0040a1a0 + 15, 0x0824748b);		// MOV ESI, dword ptr [ESP + 0x08] (move second param into ESI, which future calls expect for some reason)
	patchByte(0x0040a1a0 + 19, 0xc3);	// RET
}

void patchNoMovie() {
	// nop out video playback for compatibility with systems that cannot handle directplay
	patchNop(0x004079a8, 0x004079f4 + 5 - 0x004079a8);
}

void ledgeWarpFix() {
	// clamp st(0) to [-1, 1]
	// replaces acos call, so we call that at the end
	__asm {
		ftst
		jl negative
		fld1
		fcom
		fstp st(0)
		jle end
		fstp st(0)
		fld1
		jmp end
	negative:
		fchs
		fld1
		fcom
		fstp st(0)
		fchs
		jle end
		fstp st(0)
		fld1
		fchs
	end:
		
	}

	callFunc(0x00577cd7);
}

void patchLedgeWarp() {
	patchCall(0x0049f1dc, ledgeWarpFix);
}

// fixed rendering function for the opaque side of the rendering pipeline
// respects a material flag 0x40 that appears to denote objects that require single sided rendering, while others draw un-culled
void __cdecl fixedDrawWorldAgain(int16_t param_1, int param_2) {
	int32_t *lastVertexBuffer = 0x00906760;
	int32_t *rwObj = 0x00970a8c;
	uint32_t *currentTexture = 0x0090675c;
	int *numMeshes = 0x005c94c8;
	int *meshList = 0x00901758;
	void (__cdecl *RwGetRenderState)(int, int *) = (void *)0x0055ce60;
	void (__cdecl *RwSetRenderState)(int, int) = (void *)0x0055ce10;
	void (__cdecl *SetCurrentTexture)(int **) = (void *)0x004f4320;
	void (__cdecl *SomeLightingStuff)(int, int) = (void *)0x0052fae0;
	void (__cdecl *DrawWorldMesh)(int32_t, int *) = (void *)0x004f4210;

	int32_t uVar1;
	int texture;
	int i;
	int *mesh;	// unknown struct
  
	uVar1 = *rwObj;
	int unkRenderstate;
	RwGetRenderState(8, &unkRenderstate);
	SomeLightingStuff(0,0);
	*lastVertexBuffer = 0xffffffff;
	*currentTexture = -1;
	if (param_2) {
		RwSetRenderState(0x14,1);
		i = 0;
		if (0 < *numMeshes) {
			mesh = meshList;
			texture = *currentTexture;
			while (i < *numMeshes) {
				if ((mesh[2] != 0) && ((*(uint16_t *)(*mesh + 0x1a) & param_1) == param_2)) {
					if (texture != *mesh) {
						SetCurrentTexture(mesh);
						if ((*(uint8_t *)(*mesh + 0x1a) & 0x40)) {	// if surface is marked as double sided
							//RwSetRenderState(0x14,2);	// draw single sided
						} else {
							//RwSetRenderState(0x14,1);	// draw double sided
						}
						if ((*(uint8_t *)(*mesh + 0x1a) & 0x10)) {
							if (unkRenderstate && param_2) {
								RwSetRenderState(0x8,1);
								param_2 = 0;
							}
						} else {
							RwSetRenderState(0x8,0);
							param_2 = 1;
						}
					}
					DrawWorldMesh(*rwObj, mesh[3]);
					texture = mesh[0];
					*currentTexture = texture;
				}
				i = i + 1;
				mesh = mesh + 5;
			}
		}
		RwSetRenderState(0x14,2);
		RwSetRenderState(0x8,1);
		
	} else {
		RwSetRenderState(0x14,2);
		i = 0;
		if (0 < *numMeshes) {
			mesh = meshList;
			texture = *currentTexture;
			while (i < *numMeshes) {
				if ((mesh[2] != 0) && ((*(uint16_t *)(*mesh + 0x1a) & param_1) == 0)) {
					int materialSingleSided = 0;
					if (texture != *mesh) {
						SetCurrentTexture(mesh);
						if ((*(uint8_t *)(*mesh + 0x1a) & 0x40)) {	// if surface is marked as single sided
							RwSetRenderState(0x14,2);	// draw single sided
							materialSingleSided = 1;
						} else {
							RwSetRenderState(0x14,1);	// draw double sided
						}
					}
					DrawWorldMesh(*rwObj, mesh[3]);
					texture = mesh[0];
					*currentTexture = texture;
				}
				i = i + 1;
				mesh = mesh + 5;
			}
		}
		RwSetRenderState(0x14,2);
		RwSetRenderState(0x8,1);
	}
	return;
}

struct rw3DVertex {
	float position[3];
	float normal[3];
	uint32_t color;
	float u;
	float v;
};

void transparentizeShadow(struct rw3DVertex *verts, uint32_t numVerts, void *matrix, uint32_t flags) {
	void (__cdecl *RwIm3DTransform)(struct rw3DVertex *, uint32_t, void *, uint32_t) = (void *)0x00561010;
	void (__cdecl *RwGetRenderState)(int, int *) = (void *)0x0055ce60;
	void (__cdecl *RwSetRenderState)(int, int) = (void *)0x0055ce10;

	//printf("DRAWING SHADOW START\n");

	for (int i = 0; i < numVerts; i++) {
		//printf("Vertex: POS: %f %f %f NORM: %f %f %f COLOR: 0x%08x UV: %f %f\n", verts[i].vertex[0], verts[i].vertex[1], verts[i].vertex[2], verts[i].normal[0], verts[i].normal[1], verts[i].normal[2], verts[i].color, verts[i].u, verts[i].v);
		verts[i].color = 0x60ffffff;
	}

	//printf("DRAWING SHADOW END\n");
	//printf("Drawing shadow!\n");

	// get original blend states
	int src, dst;

	RwGetRenderState(0x0a, &src);	// 0x0a - src function
	RwGetRenderState(0x0b, &dst);	// 0x0b - dst function

	// set new states
	RwSetRenderState(0x0a, 0x03);	// src color
	RwSetRenderState(0x0b, 0x09);	// dst color
	
	RwIm3DTransform(verts, numVerts, matrix, flags);

	RwSetRenderState(0x0a, src);
	RwSetRenderState(0x0b, dst);
}

void patchCullModeFix() {
	// sets the culling mode to always be NONE rather than BACK
	// fixes missing geometry in a few levels, fixes missing faces in shadows
	
	// transparent stuff
	//patchByte(0x004f4883+1, 0x01);
	// not sure
	//patchByte(0x004f4933+1, 0x01);
	// world?
	//patchByte(0x004f4953+1, 0x01);
	// not sure
	//patchByte(0x004f5c5d+1, 0x03);
	// UI text?
	//patchByte(0x004f5cae+1, 0x01);
	// UI Button Background
	//patchByte(0x004f60d8+1, 0x03);
	// Main UI Text?
	//patchByte(0x004f60f9+1, 0x01);
	// characters?
	//patchByte(0x004f9ca3+1, 0x03);
	// shadow - this doesn't seem like a graceful fix... but it seems to work
	patchByte(0x004f9d95+1, 0x01);

	patchCall(0x004f9ba2, fixedDrawWorldAgain);
	//patchCall(0x004f9bff, fixedDrawWorldAgain);
	patchCall(0x0042db88, fixedDrawWorldAgain);

	patchCall(0x005020a5, transparentizeShadow);
}

uint8_t compLevels[] = {
	0,
	0,
	0,
	1,
	0,
	0,
	1,
	0,
	1,
	0,
};

int oldLevel = 0;
int oldProgress = 0;

void retryHook() {
	int (__cdecl *IsCareerMode)(void) = (void *)0x00421540;
	if (IsCareerMode()) {
		// reset career progress
		uint32_t *career = 0x008e1e90;
		career = (*career) + 0x134;
		career = (*career) + 0x14;

		uint32_t *level = (*career) + 0x690;
		
		if (!compLevels[*level]) {
			uint32_t *goals = (*career + 0x564 + ((*level - 1) * 8));
			uint32_t *someflags = (*career + 0x5e4 + ((*level - 1) * 8));

			*someflags = 0;	// keeps the goal messages from disappearing sometimes.  don't know what that's about.

			if (*level != oldLevel) {
				oldLevel = *level;
				oldProgress = *goals;
			}

			if (*goals & oldProgress != *goals) {
				oldProgress = *goals | oldProgress;
			}

			*goals = 0;
		}
	}

	callFunc(0x00422400);	// call retry
}

void loadRequestedLevelHook() {
	int (__cdecl *IsCareerMode)(void) = (void *)0x00421540;

	if (oldLevel != 0 && !compLevels[oldLevel]) {
		// restore career progress

		uint32_t *career = 0x008e1e90;
		career = (*career) + 0x134;
		career = (*career) + 0x14;

		uint32_t *goals = (*career + 0x564 + ((oldLevel - 1) * 8));

		*goals = *goals | oldProgress;
	}

	callFunc(0x004220c0);
}

void endRunHook() {
	int (__cdecl *IsCareerMode)(void) = (void *)0x00421540;

	if (IsCareerMode() && !compLevels[oldLevel]) {
		// restore career progress

		uint32_t *career = 0x008e1e90;
		career = (*career) + 0x134;
		career = (*career) + 0x14;

		uint32_t *goals = (*career + 0x564 + ((oldLevel - 1) * 8));

		*goals = *goals | oldProgress;
	}

	callFunc(0x004216f0);	// call end run
}

void patchILMode() {
	patchDWord(0x005b801c, retryHook);
	//patchDWord(0x005b7f8c, endRunHook);
	patchDWord(0x005b7ffc, loadRequestedLevelHook);
}

void patchTrickLimit() {
	patchByte(0x004355ad, 0xeb);
}

void our_random(int out_of) {
	// first, call the original random so that we consume a value.  
	// juuust in case someone wants actual 100% identical behavior between partymod and the original game
	void (__cdecl *their_random)(int) = (void *)0x0040e4c0;

	their_random(out_of);

	return rand() % out_of;
}

void patchRandomMusic() {
	patchCall(0x004c6b37, our_random);
}

char domainStr[256];
char peerchatStr[266];
char masterServerStr[264];

void patchOnlineService(char *configFile) {
	GetPrivateProfileString("Miscellaneous", "OnlineDomain", "openspy.net", domainStr, 256, configFile);

	sprintf(peerchatStr, "peerchat.%s", domainStr);
	sprintf(masterServerStr, "master.%s", domainStr);

	patchDWord(0x0050b278 + 1, peerchatStr);
	patchDWord(0x00517845 + 1, masterServerStr);
	patchDWord(0x0051785d + 1, masterServerStr);
	patchDWord(0x0051959a + 1, masterServerStr);

	printf("Patched online server: %s\n", domainStr);
}

void safeWait(uint64_t endTime) {
	uint64_t timerFreq = SDL_GetPerformanceFrequency();
	uint64_t safetyThreshold = (timerFreq / 1000) * 3;	// 3ms

	uint64_t currentTime = SDL_GetPerformanceCounter();

	while (currentTime < endTime) {
		currentTime = SDL_GetPerformanceCounter();

		//printf("%f\n", timerAccumulator);

		if (endTime - currentTime > safetyThreshold) {
			//uint64_t bsleep = SDL_GetPerformanceCounter();
			SDL_Delay(1);
			//uint64_t asleep = SDL_GetPerformanceCounter();
			//printf("BIG yawn! %f\n", (asleep - bsleep) / ((double)timerFreq / 1000.0));
		}
	}
}

uint64_t nextFrame = 0;
uint64_t frameStart = 0;
uint64_t safeTime = 0;

void do_frame_cap() {
	uint64_t timerFreq = SDL_GetPerformanceFrequency();
	uint64_t frameTarget = timerFreq / 60;

	if (!nextFrame || nextFrame < SDL_GetPerformanceCounter()) {
		nextFrame = SDL_GetPerformanceCounter() + frameTarget;
	} else {
		safeWait(nextFrame);
		nextFrame += frameTarget;
	}
}

void *origPresent = NULL;

/*int __fastcall presentWrapper(void *d3d8dev, void *pad, void *d3d8devagain, void *srcRect, void *dstRect, void *dstWinOverride, void *dirtyRegion) {
	int (__fastcall *present)(void *, void *, void *, void *, void *, void *, void *) = (void *)origPresent;

	//do_frame_cap();

	int result = present(d3d8dev, pad, d3d8devagain, srcRect, dstRect, dstWinOverride, dirtyRegion);

	return result;
}*/

void *origCreateDevice = NULL;

int __fastcall createDeviceWrapper(void *id3d8, void *pad, void *id3d8again, uint32_t adapter, uint32_t type, void *hwnd, uint32_t behaviorFlags, uint32_t *presentParams, void *devOut) {
	int (__fastcall *createDevice)(void *, void *, void *, uint32_t, uint32_t, void *, uint32_t, uint32_t *, void *) = (void *)origCreateDevice;


	presentParams[5] = 4;

	int result = createDevice(id3d8, pad, id3d8again, adapter, type, hwnd, behaviorFlags, presentParams, devOut);

	//if (result == 0 && devOut) {
		//uint8_t *device = **(uint32_t **)devOut;

		//origPresent = *(uint32_t *)(device + 0x3c);
		//patchDWord(device + 0x3c, presentWrapper);
	//}

	return result;
}

void * __stdcall createD3D8Wrapper(uint32_t version) {
	void *(__stdcall *created3d8)(uint32_t) = (void *)0x00569d20;

	void *result = created3d8(version);

	if (result) {
		uint8_t *iface = *(uint32_t *)result;

		origCreateDevice = *(uint32_t *)(iface + 0x3c);
		patchDWord(iface + 0x3c, createDeviceWrapper);
	}
	
	return result;
}

void patchFramerateCap() {
	patchByte(0x004c0507, 0xEB);	// skip original framerate cap logic
	patchNop(0x004c04ef, 24);
	patchCall(0x004c04ef, do_frame_cap);

	patchCall(0x0054e433, createD3D8Wrapper);

	// remove sleep from main loop
	patchNop((void *)0x004c05eb, 2);
	patchNop((void *)0x004c067a, 8);
	//patchByte((void *)(0x004c067a + 1), 0x01);
}

void __fastcall doSoundCleanup(void *skatemod) {
	void (__fastcall *origCleanup)(void *) = (void *)0x004397a0;
	void (__stdcall *soundCleanup)() = (void *)0x00408c30;

	origCleanup(skatemod);
	soundCleanup();
}

void patchSoundFix() {
	// fixes sounds failing to play after several retry
	// thanks to not6 on github for finding this fix!
	
	patchCall(0x00439b1c, doSoundCleanup);
}

uint32_t rng_seed = 0;
int ILMode;
int disableTrickLimit;

void initPatch() {
	GetModuleFileName(NULL, &executableDirectory, filePathBufLen);

	// find last slash
	char *exe = strrchr(executableDirectory, '\\');
	if (exe) {
		*(exe + 1) = '\0';
	}

	char configFile[1024];
	sprintf(configFile, "%s%s", executableDirectory, CONFIG_FILE_NAME);

	int isDebug = getIniBool("Miscellaneous", "Debug", 0, configFile);

	if (isDebug) {
		AllocConsole();

		FILE *fDummy;
		freopen_s(&fDummy, "CONIN$", "r", stdin);
		freopen_s(&fDummy, "CONOUT$", "w", stderr);
		freopen_s(&fDummy, "CONOUT$", "w", stdout);
	}
	printf("PARTYMOD for THPS3 %d.%d.%d\n", VERSION_NUMBER_MAJOR, VERSION_NUMBER_MINOR, VERSION_NUMBER_PATCH);

	printf("DIRECTORY: %s\n", executableDirectory);

	//patchResolution();

	initScriptPatches();

	int disableMovies = getIniBool("Miscellaneous", "NoMovie", 0, configFile);
	if (disableMovies) {
		printf("Disabling movies\n");
		patchNoMovie();
	}

	ILMode = getIniBool("Miscellaneous", "ILMode", 0, configFile);
	if (ILMode) {
		printf("Enabling IL Mode\n");
		patchILMode();
	}

	disableTrickLimit = getIniBool("Miscellaneous", "NoTrickLimit", 0, configFile);
	if (disableTrickLimit) {
		printf("Disabling trick limit\n");
		patchTrickLimit();
	}

	patchOnlineService(configFile);

	// get some source of entropy for the music randomizer
	rng_seed = time(NULL) & 0xffffffff;
	srand(rng_seed);

	printf("Patch Initialized\n");
}

int getVersionNumberHook(void *param) {
	void (__fastcall *unk_func1)(void *, void *, void *, void *) = (void *)0x00429dc0;
	double (__cdecl *inner_get_version)(uint32_t) = (void *)0x00411e20;
	void (__cdecl *inner_set_version)(void *, char *) = (void *)0x004ce940;

	char buffer[256];

	void *unkp;
	char *idstr = 0x005b6220;
	//printf("%s\n", idstr);
	unk_func1(param, NULL, 0x005b6220, &unkp);

	if (ILMode) {
		sprintf(buffer, " %d.%d.%d - IL MODE ACTIVE", VERSION_NUMBER_MAJOR, VERSION_NUMBER_MINOR, VERSION_NUMBER_PATCH);
	} else {
		sprintf(buffer, " %d.%d.%d", VERSION_NUMBER_MAJOR, VERSION_NUMBER_MINOR, VERSION_NUMBER_PATCH);
	}

	inner_set_version(unkp, buffer);
	return 1;
}

void patchNoLauncher() {
	// prevent the setup utility from starting
	// NOTE: if you need some free bytes, there's a lot to work with here, 0x0040b9da to 0x0040ba02 can be rearranged to free up like 36 bytes.  easily enough for a function call
	// the function to load config is in there too, so that can also be taken care of now
	patchNop((void *)0x0040b9da, 7);    // remove call to run launcher
	patchInst((void *)0x0040b9e1, JMP8);    // change launcher condition jump from JZ to JMP
	patchNop((void *)0x0040b9fc, 12);   // remove call to change registry

	// TODO: rename function to make it clear that this adds patch init
	patchCall((void *)0x0040b9da, &(initPatch));
}

void patchNotCD() {
	// Make "notCD" cfunc return true to enable debug features (mostly boring)
	patchByte((void *)0x00404350, 0xb8);
	patchDWord((void *)(0x00404350 + 1), 0x01);
	patchByte((void *)(0x00404350 + 5), 0xc3);
}

void patchPrintf() {
	patchDWord((void *)0x0058d10c, printf);
}

int (__stdcall *theirbind)(SOCKET, const struct sockaddr_in *, int) = NULL;

int __stdcall bindWrapper(SOCKET socket, struct sockaddr_in *address, int namelen) {
	address->sin_addr.S_un.S_addr = INADDR_ANY;	// bind too all interfaces instead of just one

	int result = theirbind(socket, address, namelen);

	return result;
}

void patchNetworkBind() {
	uint32_t *bindaddr = 0x00519500 + 1;

	theirbind = ((int)bindaddr) + 4 + *bindaddr;

	patchCall(0x004d9f3e, bindWrapper);	// i think this is the relevant one, but it doesn't seem to do harm to patch the rest
	patchCall(0x004d9f75, bindWrapper); 
	patchCall(0x00518a32, bindWrapper);
	patchCall(0x00519500, bindWrapper);
}

__declspec(dllexport) BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpReserved) {
	// Perform actions based on the reason for calling.
	switch(fdwReason) { 
		case DLL_PROCESS_ATTACH:
			// Initialize once for each new process.
			// Return FALSE to fail DLL load.

			// install patches
			//patchNotCD();
			patchWindow();
			patchNoLauncher();
			patchIntroMovie();
			patchLedgeWarp();
			patchCullModeFix();
			patchInput();
			patchLoadConfig();
			patchScriptHook();
			patchRandomMusic();
			patchVersionNumber();
			patchFramerateCap();
			//patchPrintf();
			patchSoundFix();
			patchNetworkBind();

			break;

		case DLL_THREAD_ATTACH:
			// Do thread-specific initialization.
			break;

		case DLL_THREAD_DETACH:
			// Do thread-specific cleanup.
			break;

		case DLL_PROCESS_DETACH:
			// Perform any necessary cleanup.
			break;
	}
	return TRUE;
}

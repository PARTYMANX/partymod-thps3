#include <windows.h>

#include <stdio.h>
#include <stdint.h>

#include <SDL2/SDL.h>

#include <patch.h>
#include <global.h>
#include <input.h>
#include <config.h>
#include <script.h>

#define VERSION_NUMBER_MAJOR 1
#define VERSION_NUMBER_MINOR 0

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
	void (__cdecl *RwGetRenderState)(int, void *) = (void *)0x0055ce60;
	void (__cdecl *RwSetRenderState)(int, int) = (void *)0x0055ce10;
	void (__cdecl *SetCurrentTexture)(int **) = (void *)0x004f4320;
	void (__cdecl *SomeLightingStuff)(int, int) = (void *)0x0052fae0;
	void (__cdecl *DrawWorldMesh)(int32_t, int *) = (void *)0x004f4210;

	int32_t uVar1;
	int texture;
	int i;
	int *mesh;	// unknown struct
  
	uVar1 = *rwObj;
	SomeLightingStuff(0,0);
	*lastVertexBuffer = 0xffffffff;
	*currentTexture = -1;
	RwSetRenderState(0x14,2);
	i = 0;
	if (0 < *numMeshes) {
		mesh = meshList;
		texture = *currentTexture;
		while (i < *numMeshes) {
			if ((mesh[2] != 0) && ((*(uint16_t *)(*mesh + 0x1a) & param_1) == 0)) {
				if (texture != *mesh) {
					SetCurrentTexture(mesh);
					if ((*(uint8_t *)(*mesh + 0x1a) & 0x40)) {
						RwSetRenderState(0x14,2);
					} else {
						RwSetRenderState(0x14,1);
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
	RwSetRenderState(8,1);
	return;
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
	patchCall(0x0042db88, fixedDrawWorldAgain);
}

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
	printf("PARTYMOD for THPS3 %d.%d\n", VERSION_NUMBER_MAJOR, VERSION_NUMBER_MINOR);

	printf("DIRECTORY: %s\n", executableDirectory);

	//patchResolution();

	initScriptPatches();

	int disableMovies = getIniBool("Miscellaneous", "NoMovie", 0, configFile);
	if (disableMovies) {
		printf("Disabling movies\n");
		patchNoMovie();
	}

	printf("Patch Initialized\n");
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

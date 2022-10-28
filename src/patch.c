#include <patch.h>

#include <stdint.h>
#include <windows.h>

void patchNop(void *addr, size_t size) {
    DWORD oldProtect;

    VirtualProtect(addr, size, PAGE_EXECUTE_READWRITE, &oldProtect);
    memset(addr, NOP, size);
    VirtualProtect(addr, size, oldProtect, &oldProtect);
}

void patchInst(void *addr, enum instruction inst) {
    DWORD oldProtect;

    VirtualProtect(addr, 1, PAGE_EXECUTE_READWRITE, &oldProtect);
    *(uint8_t *)addr = inst;
    VirtualProtect(addr, 1, oldProtect, &oldProtect);
}

void patchByte(void* addr, uint8_t val) {
    DWORD oldProtect;

    VirtualProtect(addr, 1, PAGE_EXECUTE_READWRITE, &oldProtect);
    *(uint8_t *)addr = val;
    VirtualProtect(addr, 1, oldProtect, &oldProtect);
}

void patchDWord(void *addr, uint32_t val) {
    DWORD oldProtect;

    VirtualProtect(addr, 1, PAGE_EXECUTE_READWRITE, &oldProtect);
    *(uint32_t *)addr = val;
    VirtualProtect(addr, 1, oldProtect, &oldProtect);
}

void patchFloat(void *addr, float val) {
    DWORD oldProtect;

    VirtualProtect(addr, 1, PAGE_EXECUTE_READWRITE, &oldProtect);
    *(float *)addr = val;
    VirtualProtect(addr, 1, oldProtect, &oldProtect);
}

void patchCall(void *addr, void *func) {
    DWORD oldProtect;

    VirtualProtect(addr, 1, PAGE_EXECUTE_READWRITE, &oldProtect);
    *(uint8_t *)addr = CALL;
    *(uint32_t *)((uint8_t *)addr + 1) = (uint32_t)func - (uint32_t)addr - 5;
    VirtualProtect(addr, 1, oldProtect, &oldProtect);
}

void patchThisToCdecl(void *addr, void *func) {
    // convenience function that takes 'this' and calls a c function with a pointer to it - results in 7 bytes written
    DWORD oldProtect;

    VirtualProtect(addr, 1, PAGE_EXECUTE_READWRITE, &oldProtect);

    // PUSH ECX (0x51)
    *((uint8_t *)addr) = 0x51;

    // CALL (func)
    *((uint8_t *)addr + 1) = CALL;
    *(uint32_t *)((uint8_t *)addr + 2) = (uint32_t)func - (uint32_t)((uint8_t *)addr + 1) - 5;
    
    // POP ECX (0x59)
    *((uint8_t *)addr + 6) = 0x59;

    VirtualProtect(addr, 1, oldProtect, &oldProtect);
}

void callFunc(void *addr) {
    // calls function with the assumption of it returning and accepting void
    void (*fp)() = addr;
    fp();
}

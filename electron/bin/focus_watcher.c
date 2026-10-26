#define _WIN32_WINNT 0x0600
#include <windows.h>
#include <stdio.h>
#include <string.h>

static int s_lastState = -1;

void checkAndReport(HWND hwnd) {
    if (hwnd == NULL) return;
    char className[256] = {0};
    GetClassNameA(hwnd, className, sizeof(className) - 1);
    
    // Check if foreground window is League of Legends in-game client
    // League game window class is "RiotWindowClass"
    int isLeague = (strcmp(className, "RiotWindowClass") == 0);
    
    if (isLeague != s_lastState) {
        s_lastState = isLeague;
        if (isLeague) {
            printf("LEAGUE_FOCUSED\n");
        } else {
            printf("LEAGUE_BLURRED\n");
        }
        fflush(stdout);
    }
}

void CALLBACK WinEventProc(HWINEVENTHOOK hWinEventHook, DWORD event, HWND hwnd, LONG idObject, LONG idChild, DWORD dwEventThread, DWORD dwmsEventTime) {
    if (event == EVENT_SYSTEM_FOREGROUND && hwnd != NULL) {
        checkAndReport(hwnd);
    }
}

int main(int argc, char* argv[]) {
    // Initial foreground check
    HWND initial = GetForegroundWindow();
    checkAndReport(initial);

    // Event-driven foreground hook: 0% CPU, 0ms latency, triggered instantly on Alt-Tab
    HWINEVENTHOOK hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_FOREGROUND,
        NULL,
        WinEventProc,
        0,
        0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS
    );

    if (!hook) {
        return 1;
    }

    // Windows message loop
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    UnhookWinEvent(hook);
    return 0;
}

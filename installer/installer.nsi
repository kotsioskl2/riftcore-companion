; ==============================================================================
; RiftCore Companion - Professional Modern Windows NSIS Installer Script
; Features: Branded Modern UI 2, Update Detection, In-Installer Update Notes,
;           Desktop/Start Menu Shortcuts, Clean Uninstaller, Solid LZMA Compression
; ==============================================================================

Unicode true
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

!define PRODUCT_NAME "RiftCore Companion"
!ifndef PRODUCT_VERSION
  !define PRODUCT_VERSION "1.2.14"
!endif
!define PRODUCT_PUBLISHER "RiftCore Team"
!define PRODUCT_WEB_SITE "https://github.com/kotsioskl2/riftcore-companion"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\RiftCore.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKCU"

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\installer_output\RiftCore-Setup-${PRODUCT_VERSION}.exe"
InstallDir "$LOCALAPPDATA\Programs\RiftCore"
InstallDirRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel user

; Global Variables
Var IS_UPDATE
Var PREV_VERSION

; Visual Branding & Assets
!define MUI_ABORTWARNING
!define MUI_ICON "..\public\icon.ico"
!define MUI_UNICON "..\public\icon.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "installerHeader.bmp"
!define MUI_HEADERIMAGE_RIGHT
!define MUI_WELCOMEFINISHPAGE_BITMAP "installerSidebar.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "installerSidebar.bmp"

; ------------------------------------------------------------------------------
; Installer Pages
; ------------------------------------------------------------------------------

; 1. Welcome Page (with dynamic Update detection message)
!define MUI_WELCOMEPAGE_TITLE_3LINES
!define MUI_PAGE_CUSTOMFUNCTION_PRE WelcomePage_Pre
!insertmacro MUI_PAGE_WELCOME

; 2. Software License Agreement Page (MIT License)
!define MUI_LICENSEPAGE_CHECKBOX
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"

; 3. Release Highlights / What's New Page
!define MUI_PAGE_HEADER_TEXT "Release Highlights"
!define MUI_PAGE_HEADER_SUBTEXT "What's new and improved in Version ${PRODUCT_VERSION}"
!define MUI_LICENSEPAGE_TEXT_TOP "Here is what is new in Version ${PRODUCT_VERSION}:"
!define MUI_LICENSEPAGE_TEXT_BOTTOM "Review the highlights above, then click Continue to proceed."
!define MUI_LICENSEPAGE_BUTTON "Continue >"
!insertmacro MUI_PAGE_LICENSE "UPDATE_NOTES.txt"

; 4. Installation Directory Page
!define MUI_PAGE_CUSTOMFUNCTION_PRE DirectoryPage_Pre
!insertmacro MUI_PAGE_DIRECTORY

; 4. Installation Progress Page
!insertmacro MUI_PAGE_INSTFILES

; 5. Finish Page
!define MUI_FINISHPAGE_RUN "$INSTDIR\RiftCore.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Launch RiftCore Companion"
!define MUI_FINISHPAGE_LINK "Visit RiftCore Project on GitHub"
!define MUI_FINISHPAGE_LINK_LOCATION "${PRODUCT_WEB_SITE}"
!insertmacro MUI_PAGE_FINISH

; ------------------------------------------------------------------------------
; Uninstaller Pages
; ------------------------------------------------------------------------------
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

; ------------------------------------------------------------------------------
; Pre-page Callbacks & Update Detection
; ------------------------------------------------------------------------------

Function .onInit
  ; Detect existing installation
  ReadRegStr $PREV_VERSION ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion"
  ReadRegStr $0 ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_DIR_REGKEY}" ""

  ${If} $PREV_VERSION != ""
    StrCpy $IS_UPDATE "1"
    ; If existing directory found, use it as default
    ${If} $0 != ""
      ${GetParent} "$0" $INSTDIR
    ${EndIf}
  ${Else}
    StrCpy $IS_UPDATE "0"
  ${EndIf}
FunctionEnd

Function WelcomePage_Pre
  ${If} $IS_UPDATE == "1"
    !insertmacro MUI_HEADER_TEXT "Update Detected" "Upgrading RiftCore Companion to v${PRODUCT_VERSION}"
  ${EndIf}
FunctionEnd

Function DirectoryPage_Pre
  ; If it is an update, skip folder selection to make update 1-click seamless
  ${If} $IS_UPDATE == "1"
    Abort
  ${EndIf}
FunctionEnd

; ------------------------------------------------------------------------------
; Main Installation Section
; ------------------------------------------------------------------------------

Section "MainSection" SEC01
  ; Terminate running instance if open
  DetailPrint "Checking for running instances of RiftCore..."
  nsExec::Exec 'taskkill /F /IM RiftCore.exe'

  SetOutPath "$INSTDIR"
  SetOverwrite on

  DetailPrint "Extracting core application binaries..."
  File "..\RiftCore_Rust_Portable\RiftCore.exe"
  File "..\RiftCore_Rust_Portable\WebView2Loader.dll"
  File "..\public\icon.ico"
  File "..\LICENSE"
  File "UPDATE_NOTES.txt"

  ; Create Uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Create Start Menu Shortcuts
  DetailPrint "Creating Start Menu shortcuts..."
  CreateDirectory "$SMPROGRAMS\RiftCore Companion"
  CreateShortCut "$SMPROGRAMS\RiftCore Companion\RiftCore Companion.lnk" "$INSTDIR\RiftCore.exe" "" "$INSTDIR\icon.ico" 0
  CreateShortCut "$SMPROGRAMS\RiftCore Companion\Update Notes.lnk" "$INSTDIR\UPDATE_NOTES.txt" "" "" 0
  CreateShortCut "$SMPROGRAMS\RiftCore Companion\License (MIT).lnk" "$INSTDIR\LICENSE" "" "" 0
  CreateShortCut "$SMPROGRAMS\RiftCore Companion\Uninstall RiftCore.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe" 0

  ; Create Desktop Shortcut
  DetailPrint "Creating Desktop shortcut..."
  CreateShortCut "$DESKTOP\RiftCore Companion.lnk" "$INSTDIR\RiftCore.exe" "" "$INSTDIR\icon.ico" 0

  ; Register in Windows Add/Remove Programs (Apps & Features)
  DetailPrint "Registering RiftCore in Windows System..."
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\RiftCore.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\icon.ico"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "HelpLink" "${PRODUCT_WEB_SITE}/issues"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1

  DetailPrint "Installation of RiftCore Companion completed successfully."
SectionEnd

; ------------------------------------------------------------------------------
; Uninstallation Section
; ------------------------------------------------------------------------------

Section Uninstall
  DetailPrint "Closing running instances of RiftCore..."
  nsExec::Exec 'taskkill /F /IM RiftCore.exe'

  ; Remove Shortcuts
  DetailPrint "Removing shortcuts..."
  Delete "$DESKTOP\RiftCore Companion.lnk"
  Delete "$SMPROGRAMS\RiftCore Companion\RiftCore Companion.lnk"
  Delete "$SMPROGRAMS\RiftCore Companion\Update Notes.lnk"
  Delete "$SMPROGRAMS\RiftCore Companion\Uninstall RiftCore.lnk"
  RMDir "$SMPROGRAMS\RiftCore Companion"

  ; Remove Installed Files
  DetailPrint "Removing application files..."
  Delete "$INSTDIR\RiftCore.exe"
  Delete "$INSTDIR\WebView2Loader.dll"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\UPDATE_NOTES.txt"
  Delete "$INSTDIR\uninstall.exe"

  ; Remove WebView2 User Data cache
  RMDir /r "$INSTDIR\RiftCore.exe.WebView2"
  RMDir "$INSTDIR"

  ; Delete Registry Keys
  DetailPrint "Cleaning registry entries..."
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_DIR_REGKEY}"
SectionEnd
